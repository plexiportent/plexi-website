-- This file should undo anything in `up.sql`
ALTER TABLE `posts` DROP COLUMN `author`;

DROP TABLE IF EXISTS `users`;
