# Newsletter Demo
**Warning: This s just a demo for now**
## Container Planning
![docs/archtitecture.png](docs/architecture.png)
### used Software
| Function | Software |
|-------|-----|
| Webserver| NGINX? |
| DB | PostgreSQL |
| DB library for Rust | [SQLx](https://crates.io/crates/sqlx) |
| Mail library | [lettre](https://crates.io/crates/lettre)? |
### E-Mail verification:
1. E-Mail format input validation(lib.rs)
2. [Check if it exists](https://crates.io/crates/check-if-email-exists)
3. Verification link
After all these steps were successfull, the address is written to the DB
## Project structure
- docker/  # All Container related files
  - subscriber/  # Conatains *subscriber* Container
    - Dockerfile # Build the Container for *subscriber*
  - mailbot/     # Conatains *mailbot* Container
    - Dockerfile # Build the Container for *mailbot*
- src
  - src/
    - subscriber.rs  # Takes Input and puts it in a DB  
    - mailbot.rs     # Automatically sends E-Mails to subscribed Inboxes
  - lib.rs       # Shared functionality between subscriber and mailbot(Input Validatio lays here)
- init-db  # Database config
  - 1-init.sql # Creates the needed table
  - 2-users.sh # Creates Postgres users with minimal permissions
  - 3-update-users.sh # Updates User Passwords, in case they changes
- Cargo.toml # Rust project config
- Cargo.lock # Dependecy lockfile
- compose.yaml # configures all needed Containers
- .secrets/  # not commited. Contains secrets for the Containers(e.g. DB Password)
- .gitignore

## Deploy
### Prerequisites
1. create DB secrets:
  - `.secrets/pg-subscriber` # Postgres-password for *subscriber*
  - `.secrets/pg-mailbot` # Postgres-password for *mailbot*
  - `.secrets/postgres-master-password` # Postgres admin password

You can do that by typing `openssl rand -hex 32 > *filename*`

2. Ensure correct SELinux label: `container_file_t` for all secrets and init-db contents
3. run `docker compose up`


## Project TODO:
- [ ] Input verification
  - [x] Check if email exists
  - [ ] Validation E-mail
- [x] Database setup
- [x] Allow Password rotation
- [ ] mailbot
- [ ] Webserver setup
- [x] Better Error-handling
- [ ] Extenting tests (for lib crate)
- [ ] Extending Project Docs/Comments
- [ ] Copy only needed files into Build Containers in `Dockerfile`
