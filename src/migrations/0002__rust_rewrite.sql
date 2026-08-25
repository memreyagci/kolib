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
-- In kolib, data from to-be-dropped fields are fetched before running this
-- migration, and then inserted to the new tables thereafter.
CREATE TABLE IF NOT EXISTS `twitter_dm_reactions` (
  `id` TEXT PRIMARY KEY NOT NULL,
  `main_id` TEXT NOT NULL,
  `sender_id` TEXT NOT NULL,
  `reaction_key` TEXT NOT NULL,
  `event_id` TEXT NOT NULL,
  `created_at` INTEGER NOT NULL,
  FOREIGN KEY(`main_id`) REFERENCES `twitter_direct_messages`(`id`) ON UPDATE NO action ON DELETE cascade
);
CREATE TABLE IF NOT EXISTS `twitter_dm_edit_history`(
  `id` TEXT PRIMARY KEY NOT NULL,
  `main_id` TEXT NOT NULL,
  `edited_text` TEXT NOT NULL,
  `created_at_sec` TEXT NOT NULL,
  FOREIGN KEY(`main_id`) REFERENCES `twitter_direct_messages`(`id`) ON UPDATE NO action ON DELETE cascade
);

ALTER TABLE `twitter_direct_messages` DROP COLUMN `reactions`;
ALTER TABLE `twitter_direct_messages` DROP COLUMN `edit_history`;

ALTER TABLE `twitter_direct_messages_attachments` RENAME TO `twitter_dm_attachments`;
ALTER TABLE `twitter_dm_attachments` RENAME COLUMN `message_id` TO `main_id`;

-- No need for ordinal since twitter_dm_attachments.id is uuidv7, thus we know the insertion order.
DROP INDEX `twitter_dm_attachment_message_ordinal_unique`;
ALTER TABLE `twitter_dm_attachments` DROP COLUMN `ordinal`;

-- accounts table accepted names with blank characters, this change prevents that.
PRAGMA foreign_keys = OFF;

ALTER TABLE accounts RENAME TO accounts_old;

CREATE TABLE accounts (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL CHECK(TRIM(name) <> ''),
	`platform` text NOT NULL,
	`user_id` text
);

INSERT INTO accounts (id, name, platform, user_id)
SELECT
   id,
  CASE
    WHEN TRIM(name) = '' THEN 'unnamed_account'
    ELSE name
  END,
  platform,
  user_id
FROM accounts_old;

DROP TABLE accounts_old;

PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;
