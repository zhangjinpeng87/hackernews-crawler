# Hackernews Crawler

This is a hackernews crawler which grab the latest stories and comments, store them to a MySQL compatibale database like MySQL or TiDB Cloud.

## Usage
```
crawler <database-host> <database-port> <hackernews-hub>
```
```
crawler tidb-cloud-connection-addr 4000 https://hacker-news.firebaseio.com/v0
```
Before you run the crawler you need make sure get the database ready by running steps below:

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
```
And then create user `newscrawler`:
```
CREATE USER newscrawler@'%' IDENTIFIED BY 'newscrawler';
GRANT ALL PRIVILEGES ON hackernews.* TO newscrawler@'%';
FLUSH PRIVILEGES;
```

Get the current maxitem from hackernews:
```
curl --request GET \
--url 'https://hacker-news.firebaseio.com/v0/maxitem.json?print=pretty' \
--data '{}'
```
  
And then insert the current maxitem id to table `maxitem`.
```
INSERT INTO maxitem values (1, {current-maxitem-id})
```
If you want to grab events start from a past time like 30 days' before, you can insert a relatively smaller item id in table `maxitem`.

Now you can run the crawler to grab hackernews news/stories/comments to your database.

