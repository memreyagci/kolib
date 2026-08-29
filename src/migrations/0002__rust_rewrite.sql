-- Switch from drizzle migration kolib's custom migration table.
DROP TABLE `__drizzle_migrations`;
CREATE TABLE IF NOT EXISTS `kolib_migrations` (
  `version` INTEGER PRIMARY KEY,
  `title` TEXT NOT NULL,
  `checksum` TEXT NOT NULL, -- SHA-256 hash of the .sql file
  `applied_at` INTEGER DEFAULT CURRENT_TIMESTAMP
  );

-- Create separate tables for these columns, so there is less internal
-- logic for dealing with array/json-like columns. Moreover, it will be easier
-- to deal with merging an existing dataset type in an account with a new one in
-- the future.
CREATE TABLE twitter_dm_reactions (
  main_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  reaction_key TEXT NOT NULL,
  created_at INTEGER NOT NULL,

  PRIMARY KEY (main_id, event_id),

  FOREIGN KEY (main_id)
      REFERENCES twitter_direct_messages(id)
      ON DELETE CASCADE
);
INSERT INTO twitter_dm_reactions (
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
  json_extract(reaction.value, '$.reactionKey'),
  unixepoch(json_extract(reaction.value, '$.createdAt'))
FROM twitter_direct_messages AS message,
  json_each(message.reactions) AS reaction;


CREATE TABLE twitter_dm_edit_history (
  main_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  edited_text TEXT NOT NULL,
  created_at_sec TEXT NOT NULL,

  PRIMARY KEY (main_id, ordinal),

  FOREIGN KEY (main_id)
      REFERENCES twitter_direct_messages(id)
      ON DELETE CASCADE
  );

INSERT INTO twitter_dm_edit_history (
  main_id,
  ordinal,
  edited_text,
  created_at_sec
)
SELECT
  message.id,
  CAST(edit.key AS INTEGER),
  json_extract(edit.value, '$.editedText'),
  json_extract(edit.value, '$.createdAtSec')
FROM twitter_direct_messages AS message,
  json_each(message.edit_history) AS edit;

ALTER TABLE `twitter_direct_messages` DROP COLUMN `reactions`;
ALTER TABLE `twitter_direct_messages` DROP COLUMN `edit_history`;

ALTER TABLE `twitter_direct_messages_attachments` RENAME TO `twitter_dm_attachments`;
ALTER TABLE `twitter_dm_attachments` RENAME COLUMN `message_id` TO `main_id`;

-- No need for ordinal since twitter_dm_attachments.id is uuidv7, thus we know the insertion order.
DROP INDEX `twitter_dm_attachment_message_ordinal_unique`;
ALTER TABLE `twitter_dm_attachments` DROP COLUMN `ordinal`;

CREATE INDEX `twitter_dm_attachments_main_id_id`
  ON `twitter_dm_attachments` (`main_id`, `id`);
