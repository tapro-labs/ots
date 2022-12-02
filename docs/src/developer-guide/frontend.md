# Frontend

### Built with

- [Vue 3](https://vuejs.org)
- [Vite](https://vitejs.dev)
- [Typescript](https://www.typescriptlang.org/)
- [Vue Query](https://github.com/TanStack/query/tree/main/packages/vue-query)
- [Tailwind CSS](https://tailwindcss.com/)
- [Daisy UI](https://daisyui.com/)
- [Epic Spinners](https://epic-spinners.epicmax.co/)

### Essential Tools

#### Easy

Instead of installing and managing node and yarn binaries, we recommend using [Volta](https://volta.sh)

---

#### Hard

- [NodeJS 14](https://nodejs.org/download/release/v14.21.1/)
- [Yarn 1.22](https://yarnpkg.com/)

### Setup 

- Install all [Essential Tools](#essential-tools)
- Install dependencies in root via `yarn`
- Go to the frontend directory `cd ./frontend`
- Copy `.env.example` to `.env`
  - Replace `VITE_OTS_URL` with the backend url of the secret server. See [Backend Tutorial](./backend.md#setup)
  - Replace `VITE_SLACK_CONNECT_QUERY_PARAMS` with `client_id=<slack_client_id>&scope=&user_scope=chat:write,channels:read,users:read&redirect_uri=https://<backend_server_host>/integrations/slack/webhook` by replacing `<slack_client_id>` and `<backend_server_host>` with your credentials. Here is the [Slack Guide](./how-to-integrate-with-slack.md) to create your slack application
  - The reason values in `.env.example` are with `{{}}` is explained in [Docker Tutorial](./docker.md)
- Run `yarn dev` your browser will be opened with the FE application opened. **(Default port is 3000)**
