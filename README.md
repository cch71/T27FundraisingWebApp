# About the Cloud based Server-less Fundraising Order System for Troop 27
The original goals for this system where to
* Allow the scouts to enter in mulch orders
* Be able to enter the orders from their devices in the field and replace the paper system.
* Track the scout time tracking for delivery
* Calculate financial information based on profits/other variables
* Utilize the modern server-less technologies so as to not require a constant on compute resource

# High Level Diagram

![FundraisingAppArchitecture drawio](https://user-images.githubusercontent.com/349492/177050223-5e865ffa-2360-41b6-808c-0bf00e1d4876.svg)

# Fundraising App (PWA)

From the beginning the concept for this new system was to be something the Scouts could use from their devices to make sales.  Rather than create individual mobile device apps the PWA format was chosen. The other design goal was that this be a static web app so that all the dynamic functionality happens in the browser.  This eliminates the need to have a server backend to dynamically generate the pages.

Main categories of functionality:
* User
  * Order Entry
  * User Summary/Standings
  * Mulch Spreading Status
  * Order Area Saturation (TODO)
* Admin Functionality
  * Adjust order for any user
  * Enter new orders for a user
  * Reset Fundraiser for a new year (TODO)
  * Change Fundraiser Variables (TODO)
  * Delivery user time tracking
  * Allocation adjustment

## Why Rust
Having done web development for many years I have come to swear by Typescript for large complexity projects.  Originally the first iteration was built using Gatsby however while Gatsby supported Typescript it did not easily support the type safety compilation and so maintenance was problematic and in general the system was fragile as any medium complexity javascript project often is.  There was also an expectation that it was easier for contributors to pickup javascript/typescript.  This turned out to not be true either.  To that end the 2nd version of this project was switched to the [Yew Rust Framework](https://yew.rs/).  This increased performance for app compilation, performance in the client, true type validation, easier bug fixes and feature additions.  The tradeoff is as the app gained functionality load time while not as of yet problematic did increase.  At some point when this does get to be problematic then splitting up the app into different WASM modules will solve this issue.

## Being served from AWS Amplify
We are using [AWS Amplify](https://aws.amazon.com/amplify) to serve the web app from.  This service automatically generates the TLS certificate and handles the CDN functionality.  Because this is a static page web app we are only using the AWS Amplify service to build and publish.  There is no need for the more expensive server backend to server dynamic data.  The other functionality AWS Amplify gives us is that it has a webhook setup for the repo so that when code is checked into the main branch it triggers the build/publish process in AWS Amplify.  A TODO would be to figure out how to have it trigger on the published tag releases instead of main branch to make release more obvious.

## Authentication
Authentication is handled by [Auth0](https://auth0.com/).  I am intentionally going to leave off the details because it is pretty standard stuff if you understand security and if you don't then don't need the attack vector to be too easy:)

# The Backend
## AWS API Gateway/AWS Lambda

There is nothing special about the API Gateway other than exposing an AWS Lambda required it at the time this app was developed.  The first iteration used more individual REST api's to request/submit data.  This ultimately lead to a lot of complexity and calls to the API.  The second iteration is now using GraphQL queries with a minimal number of AWS API Gateway requests.

The AWS Lambda is written in the Go language.  The first iteration was written in python however not being a type safe language this also lead to some maintenance issues.  Go being a compilable language also meant that this was ultimately cheaper than the python implementation.   While python has faster cold start times, because it is in an interpreted language and in general slower meant that the Go compiled version was actually faster for doing more which equated to less lambda compute time which was cheaper. 

The functionality of the Lambda is to convert the GraphQL request into actionable queries to the backend database.  The GraphQL also gives a schema that is easy to validate that the incoming request is in fact valid GraphQL.  It also provides a standard that can be used to abstract the request from the data source backend.  The first iteration involved REST apis with a one to one query against the AWS DynamoDB.  When it was determined that CockroachDB was a cheaper/better alternative it would have required changing all the different REST/Backing Lambada functions.  Switching to GraphQL allowed us to reduce this to a single Lambda, which more efficiently used the Lambda services, but also will allow us to switch datasources in the future without having to change the API interface.

## Database Backend
[CockroachDB](https://www.cockroachlabs.com/) is a cloud based database solution.  They have provided a PostgreSQL compatible interface so that it works well with any PostgreSQL driver available.  While DynamoDB provided some great functionality ultimately our application would benefit better from the SQL based database.  This also allows us to avoid vendor lock-in with AWS Dynamo.  There are other cloud based providers that have SQL based cloud DB access. If there is a need in the future to switch databases again, because we are using GraphQL as our interface, it should be trivial to switch to an alternative SQL DB.

# CLI Utility
Currently there isn't a user interface from the webapp to change the fundraiser system variables. However the GraphQL in the API does exists and can be controlled from the command line utility.  That said it is only meant to be used by the SuperAdmin until the user interface is available in the webapp for all admin's
