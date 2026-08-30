PRAGMA legacy_alter_table = OFF;

-- Switch from drizzle migration kolib's custom migration table.
DROP TABLE `__drizzle_migrations`;

CREATE TABLE IF NOT EXISTS `kolib_migrations` (
  `version` INTEGER PRIMARY KEY,
  `title` TEXT NOT NULL,
  `checksum` TEXT NOT NULL, -- SHA-256 hash of the .sql file
  `applied_at` INTEGER DEFAULT CURRENT_TIMESTAMP
);

-- Create a new table for Twitter DMs, to change "created_at" type
-- from INTEGER to TEXT, since I've decided to make as less conversions
-- as possible upon insertions, so that bugs fixes are not DB migrations,
-- but mere viewer updates instead.
CREATE TABLE twitter_direct_messages_v2 (
  id TEXT PRIMARY KEY NOT NULL,
  account_id TEXT NOT NULL,
  other_user_id TEXT NOT NULL,
  conversation_id TEXT NOT NULL,
  message_create_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  recipient_id TEXT NOT NULL,
  text TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Insert everything in the old Twitter DM table to the new one, while
-- converting "created_at".
INSERT INTO
  twitter_direct_messages_v2 (
    id,
    account_id,
    other_user_id,
    conversation_id,
    message_create_id,
    sender_id,
    recipient_id,
    text,
    created_at
  )
SELECT
  id,
  account_id,
  other_user_id,
  conversation_id,
  message_create_id,
  sender_id,
  recipient_id,
  text,
  strftime(
    '%Y-%m-%dT%H:%M:%fZ',
    created_at / 1000.0,
    'unixepoch'
  )
FROM
  twitter_direct_messages;

-- Create separate tables for these columns, so there is less internal
-- logic for dealing with array/json-like columns. Moreover, it will be easier
-- to deal with merging an existing dataset type in an account with a new one in
-- the future.
CREATE TABLE twitter_dm_reactions (
  main_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  reaction_key TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (main_id, event_id),
  FOREIGN KEY (main_id) REFERENCES twitter_direct_messages_v2 (id) ON DELETE CASCADE
);

INSERT INTO
  twitter_dm_reactions (
    main_id,
    event_id,
    sender_id,
    reaction_key,
    created_at
  )
SELECT
  message.id,
  json_extract(reaction.value, '$.eventId'),
  json_extract(reaction.value, '$.senderId'),
  CASE json_extract(reaction.value, '$.reactionKey')
    WHEN '👍' THEN 'agree'
    WHEN '👎' THEN 'disagree'
    WHEN '😂' THEN 'funny'
    WHEN '❤️' THEN 'like'
    WHEN '😔' THEN 'sad'
    WHEN '😮' THEN 'surprised'
    ELSE json_extract(reaction.value, '$.reactionKey')
  END,
  strftime(
    '%Y-%m-%dT%H:%M:%fZ',
    json_extract(reaction.value, '$.createdAt') / 1000.0,
    'unixepoch'
  )
FROM
  twitter_direct_messages AS message,
  json_each(message.reactions) AS reaction;

CREATE TABLE twitter_dm_edit_history (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  edited_text TEXT NOT NULL,
  created_at_sec TEXT NOT NULL,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES twitter_direct_messages_v2 (id) ON DELETE CASCADE
);

INSERT INTO
  twitter_dm_edit_history (main_id, ordinal, edited_text, created_at_sec)
SELECT
  message.id,
  CAST(edit.key AS INTEGER),
  json_extract(edit.value, '$.editedText'),
  json_extract(edit.value, '$.createdAtSec')
FROM
  twitter_direct_messages AS message,
  json_each(message.edit_history) AS edit;

CREATE TABLE twitter_dm_attachments_v2 (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  source_kind TEXT NOT NULL CHECK (source_kind IN ('url', 'file')),
  source TEXT NOT NULL,
  PRIMARY KEY (main_id, ordinal),
  FOREIGN KEY (main_id) REFERENCES twitter_direct_messages_v2 (id) ON UPDATE NO ACTION ON DELETE CASCADE
);

INSERT INTO
  twitter_dm_attachments_v2 (main_id, ordinal, source_kind, source)
SELECT
  message_id,
  ordinal,
  CASE external
    WHEN 0 THEN 'file'
    WHEN 1 THEN 'url'
  END,
  target
FROM
  twitter_direct_messages_attachments;

DROP TABLE twitter_direct_messages_attachments;

DROP TABLE twitter_direct_messages;

ALTER TABLE twitter_direct_messages_v2
RENAME TO twitter_direct_messages;

ALTER TABLE twitter_dm_attachments_v2
RENAME TO twitter_dm_attachments;

CREATE UNIQUE INDEX twitter_dm_unique ON twitter_direct_messages (account_id, message_create_id);
