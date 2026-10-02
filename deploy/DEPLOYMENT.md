# Tencent Cloud deployment

The deployment workflow is manual and only deploys the `main` branch. It sends a source archive over SSH to the Linux server, builds the containers there, then updates the Compose stack. No server password is used; the workflow authenticates with a dedicated SSH key.

## 1. Prepare the server

Use a supported Linux server with Docker Engine and the Docker Compose plugin installed. The SSH account must be able to run `docker compose` without an interactive password prompt.

Choose a writable deployment directory, for example `/home/ubuntu/xiaban`. Set the GitHub Variable `DEPLOY_PATH` to this exact path. The deployment workflow expects the server-only environment file at `$DEPLOY_PATH/.env`; it is separate from the repository and is never uploaded by the workflow.

From your computer, in the repository root, upload the example template (replace the SSH user and host):

```sh
ssh ubuntu@YOUR_SERVER_HOST "mkdir -p /home/ubuntu/xiaban"
scp deploy/.env.production.example ubuntu@YOUR_SERVER_HOST:/home/ubuntu/xiaban/.env
ssh ubuntu@YOUR_SERVER_HOST "chmod 600 /home/ubuntu/xiaban/.env && nano /home/ubuntu/xiaban/.env"
```

In that server-side file, set strong unique values for `DATABASE_PASSWORD` and `JWT_SECRET`, and set `XIABAN_DOMAIN` to the domain pointed at this server. Open inbound TCP ports 22, 80, and 443 in the Tencent Cloud security group. Do not expose PostgreSQL or Redis ports publicly.

The first deployment creates persistent Docker volumes for PostgreSQL, Redis, and Caddy certificates. Configure separate database backups before using real user data; Docker volumes alone are not backups.

## 2. Create a dedicated deploy key

On your own computer, generate a key pair specifically for this workflow:

```sh
ssh-keygen -t ed25519 -C github-actions-xiaban -f xiaban_deploy
```

Add the contents of `xiaban_deploy.pub` to the deploy user's `~/.ssh/authorized_keys` on the server. Keep `xiaban_deploy` private; do not send it in chat or commit it to the repository.

## 3. Add GitHub repository Variables and Secrets

In **bestyize/XiangBan → Settings → Secrets and variables → Actions**, add these repository Variables:

| Name | Value |
|---|---|
| `DEPLOY_HOST` | Server public IP or SSH hostname |
| `DEPLOY_USER` | SSH deploy account |
| `DEPLOY_PATH` | Absolute writable directory, such as `/home/ubuntu/xiaban` |
| `DEPLOY_PORT` | SSH port; optional, defaults to `22` |

Add these repository Secrets:

| Name | Value |
|---|---|
| `DEPLOY_SSH_KEY` | Entire private key file `xiaban_deploy` |
| `DEPLOY_KNOWN_HOSTS` | Verified SSH host-key line for this server |

Obtain the host key from a trusted network and verify its fingerprint against the server's console before saving it. For a non-default SSH port, the known-hosts entry must use the form `[host]:port key-type key-data`.

## 4. Deploy

Open **Actions → Deploy to Tencent Cloud → Run workflow**, select `main`, and start it. The workflow checks SSH host keys, copies the source archive, builds the images on the server, and runs `docker compose up -d`.

The workflow fails before updating services if the server-side `.env` file or Docker Compose is missing. Secrets stay out of the repository and deployment archive.