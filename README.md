# Available commands

### Env vars (.env)

#### Add this lines in your .env file :

    TWITCH_CLIENT_ID=
    TWITCH_CLIENT_SECRET=
    TWITCH_CLIENT_TOKEN=

### Generate Token (Scope is not optional)

- > cargo run token=moderation:read,clips:edit
- > cargo run token

### Get User by id

- > cargon run info-user=53380605

### Get user Stream by id

- > cargo run isonlive-user=147337430

### Get an user id by user login

- > cargo run uid=username

### Get Top Games

- > cargo run topgames

### Get Top Game

- > cargo run topgame

### Windows Build
- > cargo build --release --target=x86_64-pc-windows-msvc