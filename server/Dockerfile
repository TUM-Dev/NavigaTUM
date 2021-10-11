FROM gcr.io/distroless/cc-debian10

WORKDIR /srv/navigatum/
COPY navigatum-server .
COPY data/api_data.json data/api_data.json

EXPOSE 8080/tcp

ENV PATH=/srv/navigatum
CMD ["navigatum-server"]
