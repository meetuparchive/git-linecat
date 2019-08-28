# git linecat [![Build Status](https://travis-ci.com/meetup/git-linecat.svg?branch=master)](https://travis-ci.com/meetup/git-linecat) [![Coverage Status](https://coveralls.io/repos/github/meetup/git-linecat/badge.svg?branch=master)](https://coveralls.io/github/meetup/git-linecat?branch=master) [![](https://github.com/meetup/git-linecat/workflows/Main/badge.svg)](https://github.com/meetup/git-linecat/actions)

> 😽 a utility for transforming and categorizing git log output

## 🤔 about

The only constant in software is change which begs the question: What kind of patterns
of change occur in _your_ software project?

Git is a database of change but does not provide an interface for analyizing that change. This is where `git-linecat` can help.

## 📦 install

### 🍺 Via Homebrew

```sh
$ tap meetup/tools
$ brew install git-linecat
```

### 🏷️ Via GitHub Releases

Prebuilt binaries for OSX and Linux are available for download directly from GitHub Releases

```sh
$ curl -L \
 "https://github.com/meetup/git-linecat/releases/download/v0.0.0git-linecat-v0.0.0-$(uname -s)-$(uname -m).tar.gz" \
  | tar -xz
```

## 🤸usage

Expects input in the form

```sh
$ git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merge
```

Emits output in the form of [newline delimited json](http://ndjson.org/) for further analysis

### 👩‍🔬analyzing data

[AWS Athena](https://aws.amazon.com/athena/) makes it easy to both ask and answer questions about your json-formatted git data.

You can load git data into Athena simply by piping git log into `git-linecat` along with a repository name, then to AWS S3

```sh
$ git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merge \
	| cargo run -q -- -r your/repo \
	| aws s3 cp - s3://your-s3-bucket/linecat.json
```

In the Athena console, create a "table" for your data. A table is simply simply a pointer to an S3 bucket where your data is stored and a description of the shape of the data.

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

#### 🔎 Sample queries

##### top kinds of files by frequency of change


```sql
select ext, count(*) as cnt
from gitlog
group by ext
order by cnt desc
limit 10
```

##### top paths by frequency of change

```sql
select count(*) as cnt, path
from gitlog
group by path
order by cnt desc
limit 10
```

##### top paths introducing net additions to code

```sql
select path, sum(additions - deletions) as net_adds
from gitlog
group by path
order by net_adds desc
limit 10
```

##### top changers of code ownership

```sql
select count(*) as changes, author
from gitlog
where path = 'CODEOWNERS'
group by author
order by changes desc
limit 10
```

#### tips

You may find [these functions](https://docs.aws.amazon.com/athena/latest/ug/functions-operators-reference-section.html) helpful in authoring queries.

## 👩‍🏭 development

This is a [rustlang](https://www.rust-lang.org/en-US/) application.
Go grab yourself a copy with [rustup](https://rustup.rs/).

Meetup, Inc.