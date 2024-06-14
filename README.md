# git-stats
A CLI tool for generating fully customizable reports of git commit activities.

<img src="https://github.com/Some1and2-XC/git-stats/blob/main/examples/server.png" />

## Features

### Easily Generate Weekly Work Reports ([example](https://github.com/Some1and2-XC/git-stats/blob/main/examples/may_12-18_2024.pdf))
Have you ever needed to make a report of what you have been working on for any reason as a software dev? Needing to use programs to manually keep track of what you have been doing can be an arduous task. Especially since you already have been doing exactly that but in a way that's more reasonable: using git. What this program does is allows you to see all the work you have done but in a more legible format than the one that is default for storing your git files. If you go in the list view you can even use your browsers print functionality to make a report of all the things you have done in the week.

### Built in http server
For this project, it was decided that having a built in http server could be helpful for making reports easily accessible. At the moment, I'm 95% certain a directory traversal attack is possible so having this be a public facing endpoint isn't exactly recommended (this will be looked at and patched later.) That being said the API for starting this http server is fully customizable. The generated html is just calling a local endpoint which is set by the server and managing everything else client side. This means that creating a custom UI is as easy as just setting the -D argument to your own web files.

Ex:
```sh
# Where the -S starts the server
# and the -D sets which director to pull for the web files.
git-stats -S -D /usr/some/web/files
```
That being said, some webfiles are included by default if this isn't needed. These can also be used for creating a custom user interface.

### Starting Point Projection
When you make commits, generally the workflow is you write some code, then commit your changes. Because of this when you start working isn't actually tracked. Wouldn't it be nice if your fancy calendar generator could make some assumptions about when you started so that you get credit for all the work that you did? This program takes the total amount of lines added/removed and keeps track of the amount of time it takes on average for both of these metrics. This is so that every commit is counted.

### A git library
Because the author made the bad decision early on to write their own git parsing library into their project, there is also that included in the binary. This is currently in development and it doesn't include things like:
 - The ability to have more than one parent in a commit (so doesn't work fully with merges.)
 - The ability to work with compressed files (explained more further.)
 - Probably much more.

## The Opinionated Cli
Because there is two parts to this project (the actual program and the server) and all of it is running from one cli. The decision was made to make all the program arguments be lower case and the server arguments to be uppercase. For example setting the path to a git repo is set with `-d` while setting the path to the web files is done with `-D`.

## Known limitations
Because the git parsing is done manually, there are features that aren't supported just because they haven't been implemented as of yet. Some of the known and important ones are:

### Git packfiles aren't supported
Git packfiles are generated when your git files are compressed. While this may not happen often as this is generally done manually (unless you did because git gui told you to.) But importantly this is done when you `git clone` a project. This very unfortunately means it is a bit more involved to run this on someone elses project. A workaround however is to use a command such as [git unpack-objects](https://git-scm.com/docs/git-unpack-objects) to have access to the packfiles.

### Multiple Parents for a Commit isn't supported
When you merge commits, you actually have two parents save in a commit. While untested, my assumption is that at the moment the program will just pick the first parent and keep going. This means some work may not be accounted for.

## Todo!
 - Fix all the known limitations. Also fix all the build warnings.
 - Add a few more features, such as allowing users to set the window function for work blocks (which is 5h atm.)
 - Allow more control over the API client side, such as get <i>n</i> commits or get commits until <i>dd-mm-yyyy</i>.

## Shoutouts
<i>A series of articles that helped massively was from a [dev.to](https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8) user calebsander. They wrote the most comprehensive guide as to how git packfiles work. I would probably give up writing my implementation without them.</i>
