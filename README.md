# git linecat [![Build Status](https://travis-ci.com/meetup/git-linecat.svg?branch=master)](https://travis-ci.com/meetup/git-linecat) [![Coverage Status](https://coveralls.io/repos/github/meetup/git-linecat/badge.svg?branch=master)](https://coveralls.io/github/meetup/git-linecat?branch=master)

> 😽 a utility for transforming and categorizing git log output

## usage

Expects input in the form

```sh
$ git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merge
```

Emits output in the form of [newline delimited json](http://ndjson.org/) for further analysis

## analyzing data

[AWS Athena](https://aws.amazon.com/athena/) makes it easy to ask and answer questions about your json-formatted git data. 

You can load data into Athena simply by pipeline git log into `git-linecat` then to AWS S3

```sh
$ git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merge \
	| cargo run -q -- -r your/repo \
	| aws s3 cp - s3://your-s3-bucket/linecat.json
```

In the Athena console, create a "table" for your data. This is simply a pointer to an S3 bucket where your data is stored and a description of the shape of the data

```sql
CREATE EXTERNAL TABLE if not exists gitlog (
	repo string,
	sha string,
	author string,
	timestamp date,
	path string,
	category string,
	ext string,
	additions int,
	deletions int       
) 
ROW FORMAT SERDE 'org.openx.data.jsonserde.JsonSerDe'
LOCATION 's3://your-s3-bucket/'
```

Meetup, Inc.