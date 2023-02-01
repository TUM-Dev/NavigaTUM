# Server

Our server is architected in different microservices, each of which is responsible for a specific task.

- [main-api](/server/main-api): The main API server, which serves the API endpoints
- [calendar](/server/calendar): The calendar microservice, which scrapes and serves calendar data  
  This is separated from the server because:
  - it has virtually no shared dependencies (natural fault line)
  - this way, we can deploy the calendar-API independently of the main server.
    The Reason why this is important is, that scraping calendar entries is expensive for TUMOnline.
    => We have to be resourcefully and can't waste this scraped state by redeploying at will
    => Making this a StatefulSet instead of a Deployment makes sense
- [feedback](/server/feedback): The feedback microservice, which allows users to submit feedback  
  This is separated from the server because:
  - it has virtually no shared dependencies (natural faultline)
  - this way, we can deploy the feedback-API independently of the main server (both in time, scaling and reliability)
  - security: this way, we can increase our isolation and protect the GitHub token better ;)

## Getting started
Most of these components are independent of each other and share very few things.
The only some api endpoints share is the database, which contains the building data.

To set up the database, you will need to run the `load_api_data_to_db.py` script:

```bash
ln -sf ../data/output/ data
cd calendar && ln -sf ../../data/output/ data
cd main-api && ln -sf ../data/output/ data

python3 load_api_data_to_db.py
```