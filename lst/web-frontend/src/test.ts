import {
  ApolloClient,
  HttpLink,
  InMemoryCache,
  gql,
  split,
  useLazyQuery,
  useMutation,
} from "@apollo/client";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { getMainDefinition } from "@apollo/client/utilities";
import { createClient } from "graphql-ws";

const INCREMENT_COUNTER = gql`
  mutation {
    increment(value: 1)
  }
`;
const GET_COUNTER_VALUE = gql`
  query {
    value
  }
`;

function apolloClient(chainId, applicationId, port) {
  const wsLink = new GraphQLWsLink(
    createClient({
      url: `ws://localhost:${port}/ws`,
    })
  );

  const httpLink = new HttpLink({
    uri: `http://localhost:${port}/chains/${chainId}/applications/${applicationId}`,
  });

  const splitLink = split(
    ({ query }) => {
      const definition = getMainDefinition(query);
      return (
        definition.kind === "OperationDefinition" &&
        definition.operation === "subscription"
      );
    },
    wsLink,
    httpLink
  );

  return new ApolloClient({
    link: splitLink,
    cache: new InMemoryCache(),
  });
}

// Function to call the increment mutation
async function test() {
  console.log("test");
  const chainId =
    "8b989678cac87a890dea6fd94052a8a2e84a514b6e833ef2a2cdcc21b756956c";
  const applicationId =
    "05938fa64b00152e46b4b8e0cf589466e791fddb949877ae4574f9848cc27235";
  const port = 8080;

  let client = apolloClient(chainId, applicationId, port);

  const mut_resp = await client.mutate({
    mutation: INCREMENT_COUNTER,
  });
  console.log("Increment result:", mut_resp);

  // const val_resp = await client.query({
  //   query: GET_COUNTER_VALUE,
  // });
  // console.log("Value result:", val_resp.data);

  // try {
  //   const result = await client.mutate({
  //     mutation: INCREMENT_COUNTER,
  //   });
  //   console.log("Increment result:", result);
  //   return result;
  // } catch (error) {
  //   console.error("Error incrementing counter:", error);
  //   throw error;
  // }
}

// Run the function
test();
