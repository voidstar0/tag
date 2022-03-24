#  __tag, you're it!__
### give folders & files keywords for easy lookup

![An example of how to use tag](/assets/screenshot.png)

## Installation
```sh
$ brew tap char/tap
$ brew install char/tap/tag
```

## Usage

### Mark a path
```sh
$ tag mark <path> "tag1,tag2"
```

### Search a tag
```sh
$ tag find "tag"

# in current working directory
$ tag find -c "tag"
```

### Remove all tags from a path
```sh
$ tag unmark <path>
```

### Delete a tag from all paths
```sh
$ tag deltag <path>
```