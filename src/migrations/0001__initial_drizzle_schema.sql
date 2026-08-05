-- This table is to be able to know what datasets, e.g Twitter/X direct-messages.js,
-- have been imported to an account. So, instead of searching an account id throughout
-- all the tables, we only fetch from these.
CREATE TABLE `account_datasets` (
	`account_id` text NOT NULL,
	`dataset_type` text NOT NULL,
	PRIMARY KEY(`account_id`, `dataset_type`),
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`) ON UPDATE no action ON DELETE cascade
);

CREATE TABLE `accounts` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`platform` text NOT NULL,
	`user_id` text
);

CREATE TABLE `twitter_direct_messages` (
	`id` text PRIMARY KEY NOT NULL,
	`account_id` text NOT NULL,
	`other_user_id` text NOT NULL,
	`conversation_id` text NOT NULL,
	`message_create_id` text NOT NULL,
	`sender_id` text NOT NULL,
	`recipient_id` text NOT NULL,
	`text` text,
	`created_at` integer NOT NULL,
	`reactions` text DEFAULT '[]' NOT NULL,
	`edit_history` text DEFAULT '[]' NOT NULL,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`) ON UPDATE no action ON DELETE cascade
);

-- account_id and message_create_id pair must be unique. This will especially be useful when in the future,
-- feature of importing an existing dataset type to account and merging them is supported.
CREATE UNIQUE INDEX `twitter_dm_unique` ON `twitter_direct_messages` (`account_id`,`message_create_id`);

CREATE TABLE `twitter_direct_messages_attachments` (
	`id` text PRIMARY KEY NOT NULL,
	`message_id` text NOT NULL,
	`ordinal` integer NOT NULL,
	`external` integer NOT NULL,
	`target` text NOT NULL,
	FOREIGN KEY (`message_id`) REFERENCES `twitter_direct_messages`(`id`) ON UPDATE no action ON DELETE cascade
);

CREATE UNIQUE INDEX `twitter_dm_attachment_message_ordinal_unique` ON `twitter_direct_messages_attachments` (`message_id`,`ordinal`);

CREATE TABLE IF NOT EXISTS `__drizzle_migrations` (
  `id` SERIAL PRIMARY KEY,
  `hash` text NOT NULL,
  `created_at` numeric
  );
