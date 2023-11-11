Given a stream of request/response samples, api-warden will:
- infer the request/response schema (json schema)
- infer the query string schema

This will be done per api-identifier. An api-identifier is constructed using this template:
service|http-method|masked-uri|http-response-code

Everytime a change in schema is detected, a change event will be produced which will mark the change. It will include the before and after schemas.

```
kafka (traffic-samples) ---> api-warden ---> schema tracker
                                       |
                                        ---> kafka (schema change event)
```


# Traffic sample schema
the incoming samples should follow this schema
- source (string):      Identifier for the entity/component/service which you want to track the schema for
- uri (string):         URI of the request. Api-warden will attemp to normalize the uri. Check the URI Normalization section for mode details.
- qs (string):          Actual query string (params) in their normal format: p1=v1&p2=v2...
- req_hdr (map):        Key-Value pairs of request header. Api-warden will ignore the values.
- method (string):      HTTP method. It will be always converted to uppercase.
- req (string):         Actual body of the request. Can be empty or null
- resp_hdr (map):       Similar to req_hdr
- resp (string):        Actual body of the response. Can be empty of null
- resp_code (integer):  HTTP response code 



# Run The Benchmark

cargo bench --bench benchmark -- --profile-time=10