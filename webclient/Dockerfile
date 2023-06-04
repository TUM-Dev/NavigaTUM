FROM node:latest as build-stage
WORKDIR /app
COPY package*.json ./
RUN npm install --no-install-recommends &&  \
    apt-get update &&  \
    apt-get install rsync -y

COPY build.sh build.sh
COPY env env
COPY *.json ./
COPY *.ts ./
COPY public public
COPY *.html ./
COPY src src
ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}
ARG GIT_COMMIT_MESSAGE
ENV GIT_COMMIT_MESSAGE=${GIT_COMMIT_MESSAGE}
RUN sh build.sh

FROM nginx:latest as production-stage
COPY nginx.conf /etc/nginx/nginx.conf
RUN mkdir /app && apt update && apt upgrade -y
COPY --from=build-stage /dist /app
EXPOSE 9000

CMD ["nginx", "-g", "daemon off;"]
