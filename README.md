# TODO:
- [ ] use newlines from messages
- [ ] make it buid properly on ARM
- [ ] write a proper readme
- [ ] clean up commands
- [ ] more consistent error handling, started on branch: `ApiErrors`
- [ ] `find_artist`, `find_album`
- [ ] make `find_*` pipeable (like in `xargs`) to other commands like music bot (`@genius find asdf | -p`)
- [ ] custom templates
- [ ] `img` has a carrousel of all the possible images

# Bulding
## Locally
`docker build -t genius-bot .`

## Cross compiling to ARM
`docker build -t genius-bot-arm -f Dockerfile.arm .`

# Heroku
This project is powered by heroku.
To build it and publish use `heroku container:push worker && heroku container:release worker`.
Remember to `heroku container:login` first!
Also remember to check which discord token is selected in `.env`.
