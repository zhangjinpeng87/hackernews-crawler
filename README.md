# Hackernews Crawler

'''
crawler <database-host> <database-port> <hackernews-hub = https://hacker-news.firebaseio.com/v0>
''' 

```
CREATE DATABASE hackernews;
USE hackernews;
CREATE TABLE `items` (
  `id` int(10) NOT NULL,
  `deleted` tinyint(4) DEFAULT '0',
  `type` varchar(16) DEFAULT NULL,
  `who` varchar(255) DEFAULT NULL,
  `time` int(11) DEFAULT NULL,
  `dead` tinyint(4) DEFAULT '0',
  `kids` text DEFAULT NULL,
  `title` text DEFAULT NULL,
  `content` text DEFAULT NULL,
  `score` int(10) DEFAULT NULL,
  `url` text DEFAULT NULL,
  `parent` int(10) DEFAULT NULL,
  PRIMARY KEY (`id`) /*T![clustered_index] CLUSTERED */,
  KEY `_idx_time` (`time`)
);
CREATE TABLE `maxitem` (
  `id` int(10) NOT NULL,
  `maxid` int(10) DEFAULT NULL,
  PRIMARY KEY (`id`) /*T![clustered_index] CLUSTERED */
);

Get the current maxitem from hackernews:
> curl --request GET \
  --url 'https://hacker-news.firebaseio.com/v0/maxitem.json?print=pretty' \
  --data '{}'

INSERT INTO maxitem (1, {current-maxitem-id})
```
