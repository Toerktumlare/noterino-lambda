# noterino-lambda
the evil backend for the `noterino` app

Features:
- [x] setup build pipeline
- [x] setup deploy pipeline
- [x] build zip package
- [x] deploy to aws
- [ ] improve pipeline build time to use caching
- [ ] improve temployment pipeline to include descriptions in templates, and other good stuff
- [ ] set parameters during deploy
- [ ] `POST /document`
- [ ] `GET /document`

Pipeline:
- [x] upsert env using template

Test
- [ ] set parameters during testing
- [x] script for starting environment (function app and docker image)
- [x] provision tables in dynamoDB local
- [x] create test data in local dynamoDB
