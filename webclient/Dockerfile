FROM node:21 as build-stage
WORKDIR /app
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY package.json ./
COPY pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY . .
ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
ARG GIT_COMMIT_MESSAGE
ENV GIT_COMMIT_MESSAGE=${GIT_COMMIT_MESSAGE}
RUN NITRO_PRESET=deno-server pnpm run build


FROM denoland/deno:alpine as production-stage
RUN apk update --no-progress --quiet && apk add curl --no-progress --quiet

COPY --from=build-stage /app/.output /app/.output
COPY --from=build-stage /app/node_modules /app/node_modules
COPY --from=build-stage /app/.nuxt /app/.nuxt

ENV NUXT_HOST=0.0.0.0
ENV NUXT_PORT=3000
EXPOSE 3000

HEALTHCHECK CMD curl --fail localhost:3000/health || exit 1
CMD deno run --unstable --allow-net --allow-read --allow-env /app/.output/server/index.mjs
