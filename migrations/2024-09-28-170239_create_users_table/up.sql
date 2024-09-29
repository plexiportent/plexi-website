-- Your SQL goes here
ALTER TABLE `posts` ADD COLUMN `author` INTEGER NOT NULL;

CREATE TABLE `users`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`email` TEXT NOT NULL,
	`password_hash` TEXT
);

