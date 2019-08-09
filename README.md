# git linecat

> 😽 a utility for transforming and categorizing git log output

## usage

Expects input in the form

```sh
$ git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merge
```

Emits output in the form of [newline delimited json](http://ndjson.org/) for further analysis