# Hackernews Crawler

This is a hackernews crawler which grab the latest stories and comments, store them to a MySQL compatibale database like MySQL or [TiDB Cloud](https://tidbcloud.com/).

## Build
```
make build
```

## Usage
```
crawler <db-host> <db-port> <db-name> <db-user> <db-pwd> <hackernews-hub>
```
```
crawler tidb-cloud-connection-addr 4000 https://hacker-news.firebaseio.com/v0
```
For me, I use TiDB Cloud dev-tier as database to store data.

![aee3c3b4-228b-456f-9e9d-519af640fd46](https://user-images.githubusercontent.com/19700528/176884255-8118191e-c395-4fee-97f6-3559f70d48ec.jpeg)

I wrote some SQLs and output the result in retool.

Latest 500 new stories:
```
select who as author, time, title, score, url from hackernews.items where type="story" and title <> '' and url <> '' order by time DESC limit 500;
```

Top 20 authors in last ? days:
```
select who as author, count(*) as number_of_article from items where type="story" and who is not null and who <> '' and time > UNIX_TIMESTAMP(NOW() - INTERVAL ? day) group by author order by number_of_article desc limit 20;
```

Top 20 commenters in last ? days:
```
select who as author, count(*) as number_of_comments from items where type="comment" and who is not null and who <> '' and time > UNIX_TIMESTAMP(NOW() - INTERVAL ? day) group by author order by number_of_comments desc limit 20;
```

Number of comments for each of last 7 days:
```
select date(from_unixtime(time)) as date, count(*) as number_of_stories from items where type='comment' group by date order by date desc limit 7;
```

Number of new stories distributed by hour:
```
select hour(from_unixtime(time)) as date, count(*) as number_of_stories from items where type='story' group by date order by date desc;
```

## Preparations

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

Then you can play with these data.
