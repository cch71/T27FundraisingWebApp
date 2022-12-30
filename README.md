# About the Cloud based Server-less Fundraising Order System for Troop 27

The original goals for this system where to

- Allow the scouts to enter in mulch orders
- Be able to enter the orders from their devices in the field and replace the paper system.
- Track the scout time tracking for delivery
- Calculate financial information based on profits/other variables
- Utilize the modern server-less technologies so as to not require a constant on compute resource

# High Level Diagram

~~~mermaid
C4Context
title T27 Fundraising Web System

Person(seller, "Seller", "Fundraiser Seller")
Person(admin, "Admin", "Fundraising Admin")

System_Boundary(web, "Web") {
  System(webClient, "Web Client (PWA)")
}

System_Boundary(b2, "Serverless Cloud") {
  System(awsAmplify, "AWS Amplify", "Web Client Server")
  System(awsApiGw, "AWS API Gateway")
  System(awsLambda, "AWS Lambda")
  System_Ext(keycloak, "phaseTwo (KeyCloak)", "Authentication and IAM")
  SystemDb_Ext(db, "CockroachDB (Postgresql)")
}


Rel(awsAmplify, webClient, "Publishes")
BiRel(awsLambda, awsApiGw, "Calls")
BiRel(awsApiGw, webClient, "Uses")

BiRel(admin, webClient, "Uses")
BiRel(seller, webClient, "Uses")
BiRel(webClient, keycloak, "Authenticates")
BiRel(awsLambda, db, "SQL")
BiRel(awsApiGw, keycloak, "Verifies")

~~~


# Fundraising App (PWA)

From the beginning the concept for this new system was to be something the Scouts could use from
their devices to make sales. The decision was made to create this app as a Progressive Web App (PWA) 
rather than create individual mobile device apps. The other design goal was that this be a static web
app so that all the dynamic functionality happens in the browser.This eliminates the need to have a
server backend to dynamically generate the pages.

Main categories of functionality:

- User
  - Order Entry
  - User Summary/Standings
  - Mulch Spreading Status
  - Report generation capabilities
  - Order Area Saturation Heatmaps (TODO)
- Admin Functionality
  - Adjust order for any user
  - Enter new orders for a user
  - Reset Fundraiser for a new year
  - Change Fundraiser Variables
  - Delivery workers time tracking
  - Allocation adjustment

## Why Rust

Having done web development for many years I have come to swear by Typescript for large complex projects.
Originally the first iteration of this app was built using Gatsby however while Gatsby supported Typescript it did not easily
support the type safety compilation and so maintenance was problematic and in general the system was fragile
as any medium complexity javascript project often can become.
There was also an expectation that it was easier for contributors to pickup Javascript/Typescript.
This turned out to not be true either.  To that end the 2nd version of this project was switched to the [Yew Rust Framework](https://yew.rs/).
This increased performance for app compilation, performance in the client, true type validation, easier bug fixes and feature additions.
The tradeoff is as the app gained functionality load time did increase. However not enough to be an issue.  At some point when this
does get to be problematic then splitting up the app into different WASM modules will solve this issue.

## Being served from AWS Amplify

We are using [AWS Amplify](https://aws.amazon.com/amplify) to serve the web app from.  This service automatically
generates the TLS certificate and handles the CDN functionality.  Since this is a static page web app we are only
using the AWS Amplify service to build and publish.  There is no need for the more expensive server backend to serve dynamic data.
The other functionality AWS Amplify gives us is that it has a webhook setup for the repo so that when code is checked into
the main branch it triggers the build/publish process in AWS Amplify.  A TODO would be to figure out how to have it trigger
on the published tag releases instead of main branch to make release more obvious.

## Authentication

Authentication is handled by [Phase//](https://phasetwo.io/). Phase//(aka PhaseTwo) is a provider that is based on
[KeyCloak](https://www.keycloak.org/).  Authentication has evolved since the original implementation that started out
with a pure AWS Cognito solution. We then moved to using Okta's Auth0 service as an attempt to gain cloud vendor lock-in.
We are not using the KeyCloak based system.  This gives up greater flexibility as there are multiple KeyCloak cloud vendors or we could setup
our own.  While Auth0 hand some less than ideal limits (for the free account) so far Phase// has been perfect for our needs.
Using a KeyCloak based system also means that the admin scripts that we write can be used with other KeyCloak providers.

# The Backend

## AWS API Gateway/AWS Lambda

There is nothing special about the API Gateway other than exposing an AWS Lambda required it at the time this app was developed.
The first iteration used more individual REST api's to request/submit data.  This ultimately lead to a lot of complexity and
seperate calls to the API.  The second iteration is now using GraphQL queries with a minimal number of AWS API Gateway requests.

The AWS Lambda is also now written in the Go language.  The first iteration was written in Python3 however not being a type safe language
this also lead to some maintenance issues.  The Go language being a compilable language also meant that this was ultimately cheaper than the
Python implementation.   While Python has faster cold start times, because it is in an interpreted language and in general slower,
meant that the Go compiled version was actually faster for doing more which equated to less lambda compute time which was cheaper.

The functionality of the Lambda is to convert the GraphQL request into actionable queries to the backend database.  The GraphQL
also gives a schema that is easy to validate that the incoming request is in fact valid GraphQL.  It also provides a standard that
can be used to abstract the request from the data source backend.  The first iteration involved REST APIs with a one to one query
against the AWS DynamoDB.  When it was determined that CockroachDB was a cheaper/better alternative it would have required changing
all the different REST/Backing Lambada functions.  Switching to GraphQL allowed us to reduce this to a single Lambda, which more
efficiently used the Lambda services, but also will allow us to switch datasources in the future without having to change the API interface.

## Database Backend

[CockroachDB](https://www.cockroachlabs.com/) is a cloud based database solution.  They have provided a PostgreSQL compatible interface
so that it works well with any PostgreSQL driver available.  While DynamoDB provided some great functionality ultimately our application
would benefit better from the SQL based database.  This also allows us to avoid vendor lock-in with AWS Dynamo.  There are other cloud
based providers that have SQL (and specifically Postgresql) based cloud DB access. If there is a need in the future to switch databases
again it should be trivial to switch to an alternative database provider because we are using GraphQL as our interface.

# CLI Utility

A CLI tool written in Go was created as a tool to both debug the GraphQL APIs and create local functionality not wanting to be exposed
vis the web browser.  For security reasons we don't allow the web interface to create the authentication accounts and so the CLI utility
is the mechansim used to create accounts based on a request made via the web browser.
