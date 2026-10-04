# Tencent Cloud deployment

The deployment workflow is manual and only deploys the `main` branch. It sends a source archive over SSH to the Linux server, builds the containers there, then updates the Compose stack. No server password is used; the workflow authenticates with a dedicated SSH key.

This deployment is designed to reuse an existing host Nginx and Certbot setup. The Compose stack does not bind ports 80 or 443. It publishes the XiangBan web container only on `127.0.0.1:18080`; the existing host Nginx remains the public entry point and handles HTTPS certificates.

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

The host Nginx must have a virtual host for `XIABAN_DOMAIN` that proxies to `http://127.0.0.1:18080`. For example, add this location to the domain's existing HTTPS server block, keeping its Certbot-managed certificate directives:

```nginx
location / {
    proxy_pass http://127.0.0.1:18080;
    proxy_http_version 1.1;
    proxy_set_header Host $http_host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

If the certificate for this domain has not been issued yet, first make sure DNS points to this server and the host Nginx serves the domain over HTTP, then use the existing Certbot Nginx workflow to issue it. Test the Nginx configuration before reloading Nginx. Do not add another reverse proxy that binds ports 80 or 443.

The first deployment creates persistent Docker volumes for PostgreSQL and Redis. Configure separate database backups before using real user data; Docker volumes alone are not backups.

### Local VMware preview

For a quick preview from the Windows host without a domain or HTTPS proxy, set these values in the server-side `.env` file (replace the example IP with the VM's current VMware NAT address):

```dotenv
XIABAN_DOMAIN=192.168.164.128
PUBLIC_BASE_URL=http://192.168.164.128:18080
WEB_BIND_ADDRESS=192.168.164.128
VITE_DATA_OVERSEAS=false
```

Open `http://192.168.164.128:18080` from the host. This is intended for a local preview; other physical LAN devices cannot reach a VMware NAT address unless the VM network is changed to bridged mode or a host port forward is configured.

To avoid compiling the Kotlin backend on the VM, build its install distribution on the host with `gradlew.bat :server:installDist -x test`, then run Compose with `-f compose.yaml -f deploy/compose.vmware-prebuilt.yaml`. The overlay uses the runtime-only server Dockerfile and copies that prebuilt output into the image.

## Administrator and email configuration

The website provides `/admin` for administrators and `/login` for passwordless email registration and login. SMTP settings can be saved from the admin page and take effect immediately for verification codes and background mail workers. Use the saved configuration's test-mail button to confirm delivery before inviting users to register.

The admin UI uses a left navigation sidebar and separate pages for overview, user management (`/admin/users`), email settings, operation logs, and account security. User management loads server-side pages of up to 100 rows and supports literal searches across ID, email, nickname, companion name, and invite code. Filters include email/guest account type, onboarding status, and account creation dates in Asia/Shanghai. Account creation and email verification timestamps are shown separately because visitors are initially anonymous accounts.

User details show registration source, consent timestamps, associated-browser dates, and recent valid check-ins. Internal operator notes are limited to 2,000 characters. Note updates and their audit records are committed together. Email-setting and admin-password changes are also audited, without recording their credentials. The CSV button exports only the loaded page and neutralizes spreadsheet formulas in user-provided fields. These endpoints require an administrator session; ordinary user JWTs do not grant access.

User management also supports editing nickname, companion name/personality, city, timezone, working hours, and payday. The editor sends only changed fields, and updates are validated and audited in one transaction. Registration timestamps and verified-email identity remain part of the account's verification flow. Permanent deletion requires entering the target user ID in the UI and matching `confirmUserId` on the API. Account-dependent rows are removed through database cascades, guardian schedules are cleared, and deleted accounts fail JWT authentication. Deletion audit records retain the user number in their summary.

On first setup, the administrator username defaults to `admin` and the bootstrap password defaults to `123456`. The password also accepts the server-only `ADMIN_BOOTSTRAP_PASSWORD` override. Recreate the server container after changing the environment. The bootstrap password creates the first administrator on the first login; later logins use the password hash stored in PostgreSQL. Change the initial password from the admin page. Once the account exists, the bootstrap password can be removed from `.env`.

Administrator sessions expire after eight hours. Changing the administrator password invalidates all existing administrator sessions. The web UI never receives stored SMTP passwords; leaving the SMTP password field empty preserves the saved value. SMTP configuration is encrypted in PostgreSQL. Preserve `SETTINGS_ENCRYPTION_KEY` alongside database backups: replacing it makes previously saved credentials unreadable.

Without SMTP configuration, sending a verification code returns an explicit service-unavailable response rather than reporting a logged email as delivered. Codes expire after ten minutes, are scoped to their device account and operation, allow five wrong guesses, and can be consumed only once. Resends are limited per email, account, and IP address. Existing-email login merges guest records only when the user chooses to do so; binding a guest from the account page retains its original merge behavior.

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

The workflow fails before updating services if the server-side `.env` file or Docker Compose is missing. After deployment, check that the Compose web service is listening on `127.0.0.1:18080`, then test the domain through the existing host Nginx and Certbot-managed HTTPS setup.
