# Newsletter Demo
**Warning: This s just a demo for now**
## Container Planning
![docs/archtitecture.png](docs/architecture.png)

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
- Cargo.toml # Rust project config
- Cargo.lock # Dependecy lockfile
- compose.yaml # configures all needed Containers
- .secrets/  # not commited. Contains secrets for the Containers(e.g. DB Password)
- .gitignore
