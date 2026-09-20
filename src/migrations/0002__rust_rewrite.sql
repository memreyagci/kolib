PRAGMA legacy_alter_table = OFF;

-- Switch from drizzle migration kolib's custom migration table.
DROP TABLE `__drizzle_migrations`;

CREATE TABLE IF NOT EXISTS `kolib_migrations` (
  `version` INTEGER PRIMARY KEY,
  `title` TEXT NOT NULL,
  `checksum` TEXT NOT NULL, -- SHA-256 hash of the .sql file
  `applied_at` INTEGER DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE platforms (id TEXT PRIMARY KEY NOT NULL);

INSERT INTO
  platforms (id)
VALUES
  ('twitter');

CREATE TABLE dataset_types (id TEXT PRIMARY KEY NOT NULL);

INSERT INTO
  dataset_types (id)
VALUES
  ('messages');

-- Rebuild accounts so its platform is constrained by the platforms table and
-- (id, platform) can be referenced by data tables.
CREATE TABLE accounts_v2 (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  platform TEXT NOT NULL,
  user_id TEXT,
  UNIQUE (id, platform),
  FOREIGN KEY (platform) REFERENCES platforms (id) ON UPDATE NO ACTION ON DELETE RESTRICT
);

INSERT INTO
  accounts_v2 (id, name, platform, user_id)
SELECT
  id,
  name,
  platform,
  user_id
FROM
  accounts;

CREATE TABLE messages (
  id TEXT PRIMARY KEY NOT NULL,
  account_id TEXT NOT NULL,
  platform TEXT NOT NULL,
  conversation_id TEXT NOT NULL,
  record_id TEXT NOT NULL,
  sender TEXT NOT NULL,
  recipient TEXT,
  text TEXT,
  created_at_ms INTEGER,
  FOREIGN KEY (account_id, platform) REFERENCES accounts_v2 (id, platform) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Platform-specific identity indexes may become unnecessary if every parser
-- eventually generates conversation and record IDs with common guarantees.
CREATE UNIQUE INDEX twitter_message_identity ON messages (account_id, conversation_id, record_id)
WHERE
  platform = 'twitter';

CREATE TABLE message_reactions (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  -- Stable record identity, supplied by the platform when available or
  -- deterministically derived by the platform parser.
  record_id TEXT,
  sender TEXT,
  reaction TEXT NOT NULL,
  created_at_ms INTEGER,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES messages (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

CREATE UNIQUE INDEX message_reaction_record_identity ON message_reactions (main_id, record_id)
WHERE
  record_id IS NOT NULL;

CREATE TABLE message_edits (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  text TEXT NOT NULL,
  created_at_ms INTEGER,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES messages (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

CREATE TABLE message_file_attachments (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  filename TEXT NOT NULL,
  created_at_ms INTEGER,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES messages (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

CREATE TABLE message_link_attachments (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  url TEXT NOT NULL,
  created_at_ms INTEGER,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES messages (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

INSERT INTO
  messages (
    id,
    account_id,
    platform,
    conversation_id,
    record_id,
    sender,
    recipient,
    text,
    created_at_ms
  )
SELECT
  id,
  account_id,
  'twitter',
  conversation_id,
  message_create_id,
  sender_id,
  recipient_id,
  text,
  created_at
FROM
  twitter_direct_messages;

INSERT INTO
  message_reactions (
    main_id,
    ordinal,
    record_id,
    sender,
    reaction,
    created_at_ms
  )
SELECT
  message.id,
  CAST(reaction.key AS INTEGER),
  json_extract(reaction.value, '$.eventId'),
  json_extract(reaction.value, '$.senderId'),
  json_extract(reaction.value, '$.reactionKey'),
  CAST(
    json_extract(reaction.value, '$.createdAt') AS INTEGER
  )
FROM
  twitter_direct_messages AS message,
  json_each(message.reactions) AS reaction;

INSERT INTO
  message_edits (main_id, ordinal, text, created_at_ms)
SELECT
  message.id,
  CAST(edit.key AS INTEGER),
  json_extract(edit.value, '$.editedText'),
  CAST(
    json_extract(edit.value, '$.createdAtSec') AS INTEGER
  ) * 1000
FROM
  twitter_direct_messages AS message,
  json_each(message.edit_history) AS edit;

INSERT INTO
  message_file_attachments (main_id, ordinal, filename)
SELECT
  message_id,
  ordinal,
  target
FROM
  twitter_direct_messages_attachments
WHERE
  external = 0;

INSERT INTO
  message_link_attachments (main_id, ordinal, url)
SELECT
  message_id,
  ordinal,
  target
FROM
  twitter_direct_messages_attachments
WHERE
  external = 1;

CREATE TABLE account_datasets_v2 (
  account_id TEXT NOT NULL,
  dataset_type TEXT NOT NULL,
  PRIMARY KEY (account_id, dataset_type),
  FOREIGN KEY (account_id) REFERENCES accounts_v2 (id) ON UPDATE NO ACTION ON DELETE CASCADE,
  FOREIGN KEY (dataset_type) REFERENCES dataset_types (id) ON UPDATE NO ACTION ON DELETE RESTRICT
);

INSERT INTO
  account_datasets_v2 (account_id, dataset_type)
SELECT
  account_id,
  'messages'
FROM
  account_datasets
WHERE
  dataset_type = 'direct-messages.js';

DROP TABLE twitter_direct_messages_attachments;

DROP TABLE twitter_direct_messages;

DROP TABLE account_datasets;

DROP TABLE accounts;

ALTER TABLE accounts_v2
RENAME TO accounts;

ALTER TABLE account_datasets_v2
RENAME TO account_datasets;

CREATE TABLE contacts (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  is_me INTEGER NOT NULL DEFAULT 0 CHECK (is_me IN (0, 1))
);

-- An archive may have many identities assigned to its built-in "Me" contact,
-- but it may only have one contact that represents the archive owner.
CREATE UNIQUE INDEX contacts_single_me ON contacts (is_me)
WHERE
  is_me = 1;

CREATE TABLE message_contact_assignments (
  account_id TEXT NOT NULL,
  conversation_id TEXT NOT NULL,
  participant_key TEXT NOT NULL,
  contact_id TEXT NOT NULL,
  PRIMARY KEY (account_id, conversation_id, participant_key),
  FOREIGN KEY (account_id) REFERENCES accounts (id) ON UPDATE NO ACTION ON DELETE CASCADE,
  FOREIGN KEY (contact_id) REFERENCES contacts (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

CREATE TRIGGER message_contacts_delete_with_dataset AFTER DELETE ON account_datasets WHEN OLD.dataset_type = 'messages' BEGIN
DELETE FROM message_contact_assignments
WHERE
  account_id = OLD.account_id;

END;

CREATE TRIGGER accounts_validate_name_on_insert BEFORE INSERT ON accounts WHEN length(trim(NEW.name)) NOT BETWEEN 1 AND 100  BEGIN
SELECT
  RAISE (
    ABORT,
    'account name must be between 1 and 100 characters'
  );

END;

CREATE TRIGGER accounts_validate_name_on_update BEFORE
UPDATE OF name ON accounts WHEN length(trim(NEW.name)) NOT BETWEEN 1 AND 100  BEGIN
SELECT
  RAISE (
    ABORT,
    'account name must be between 1 and 100 characters'
  );

END;

CREATE TRIGGER messages_create_dataset AFTER INSERT ON messages BEGIN
INSERT OR IGNORE INTO
  account_datasets (account_id, dataset_type)
VALUES
  (NEW.account_id, 'messages');

END;

-- Keep a full-text index of message text for word and word-prefix searches.
CREATE VIRTUAL TABLE messages_fts USING fts5 (
  message_id UNINDEXED,
  text,
  tokenize = 'unicode61'
);

-- Index messages that were copied from the v1 table earlier in this migration.
INSERT INTO
  messages_fts (message_id, text)
SELECT
  id,
  text
FROM
  messages;

CREATE TRIGGER messages_fts_after_insert AFTER INSERT ON messages BEGIN
INSERT INTO
  messages_fts (message_id, text)
VALUES
  (NEW.id, NEW.text);

END;

CREATE TRIGGER messages_fts_after_delete AFTER DELETE ON messages BEGIN
DELETE FROM messages_fts
WHERE
  message_id = OLD.id;

END;

CREATE TRIGGER messages_fts_after_text_update AFTER
UPDATE OF text ON messages BEGIN
UPDATE messages_fts
SET
  text = NEW.text
WHERE
  message_id = OLD.id;

END;
