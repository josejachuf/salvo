Endpoint attribute macro implements OpenAPI path for the decorated function.

Macro accepts set of attributes that can be used to configure and override default values what are resolved automatically.

You can use the Rust's own `#[deprecated]` attribute on functions to mark it as deprecated and it will
reflect to the generated OpenAPI spec. Only **parameters** has a special **deprecated** attribute to define them as deprecated.

`#[deprecated]` attribute supports adding additional details such as a reason and or since version but this is is not supported in
OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
`#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.

Doc comment at decorated function will be used for _`description`_ and _`summary`_ of the path.
First line of the doc comment will be used as the _`summary`_ and the whole doc comment will be
used as _`description`_.
```
/// This is a summary of the operation
///
/// All lines of the doc comment will be included to operation description.
#[salvo_oapi::endpoint()]
fn endpoint() {}
```

# Endpoint Attributes

* `operation_id = ...` Unique operation id for the endpoint. By default this is mapped to function name.
  The operation_id can be any valid expression (e.g. string literals, macro invocations, variables) so long
  as its result can be converted to a `String` using `String::from`.

* `tag = "..."` Can be used to group operations. Operations with same tag are grouped together. By default
  this is derived from the handler that is given to [`OpenApi`][openapi]. If derive results empty str
  then default value _`crate`_ is used instead.

* `request_body = ... | request_body(...)` Defining request body indicates that the request is expecting request body within
  the performed request.

* `responses(...)` Slice of responses the endpoint is going to possibly return to the caller.

* `parameters(...)` Slice of parameters that the endpoint accepts.

* `security(...)` List of [`SecurityRequirement`][security]s local to the path operation.

# Request Body Attributes

**Simple format definition by `request_body = ...`**
* _`request_body = Type`_, _`request_body = inline(Type)`_ or _`request_body = ref("...")`_.
  The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
  With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
  [`AsSchema`][as_schema] types. _`ref("./external.json")`_ can be used to reference external
  json file for body schema. **Note!** Sapid does **not** guarantee that free form _`ref`_ is accessbile via
  OpenAPI doc or Swagger UI, users are eligible to make these guarantees.

**Advanced format definition by `request_body(...)`**
* `content = ...` Can be _`content = Type`_, _`content = inline(Type)`_ or _`content = ref("...")`_. The
  given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec
  or Map etc. With _`inline(...)`_ the schema will be inlined instead of a referenced
  which is the default for [`AsSchema`][as_schema] types. _`ref("./external.json")`_
  can be used to reference external json file for body schema. **Note!** Sapid does **not** guarantee
  that free form _`ref`_ is accessible via OpenAPI doc or Swagger UI, users are eligible
  to make these guarantees.

* `description = "..."` Define the description for the request body object as str.

* `content_type = "..."` Can be used to override the default behavior of auto resolving the content type
  from the `content` attribute. If defined the value should be valid content type such as
  _`application/json`_. By default the content type is _`text/plain`_ for
  [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and
  _`application/json`_ for struct and complex enum types.

* `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
  _`serde_json::json!`_ can parse as a _`serde_json::Value`_.

* `examples(...)` Define multiple examples for single request body. This attribute is mutually
  exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
  This has same syntax as _`examples(...)`_ in [Response Attributes](#response-attributes)
  _examples(...)_

_**Example request body definitions.**_
```text
 request_body(content = String, description = "Xml as string request", content_type = "text/xml"),
 request_body = Pet,
 request_body = Option<[Pet]>,
```

# Response Attributes

* `status = ...` Is either a valid http status code integer. E.g. _`200`_ or a string value representing
  a range such as _`"4XX"`_ or `"default"` or a valid _`http::status::StatusCode`_.
  _`StatusCode`_ can either be use path to the status code or _status code_ constant directly.

* `description = "..."` Define description for the response as str.

* `body = ...` Optional response body object type. When left empty response does not expect to send any
  response body. Can be _`body = Type`_, _`body = inline(Type)`_, or _`body = ref("...")`_.
  The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
  With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
  [`AsSchema`][as_schema] types. _`ref("./external.json")`_
  can be used to reference external json file for body schema. **Note!** Sapid does **not** guarantee
  that free form _`ref`_ is accessible via OpenAPI doc or Swagger UI, users are eligible
  to make these guarantees.

* `content_type = "..." | content_type = [...]` Can be used to override the default behavior of auto resolving the content type
  from the `body` attribute. If defined the value should be valid content type such as
  _`application/json`_. By default the content type is _`text/plain`_ for
  [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and
  _`application/json`_ for struct and complex enum types.
  Content type can also be slice of **content_type** values if the endpoint support returning multiple
 response content types. E.g _`["application/json", "text/xml"]`_ would indicate that endpoint can return both
 _`json`_ and _`xml`_ formats. **The order** of the content types define the default example show first in
 the Swagger UI. Swagger UI wil use the first _`content_type`_ value as a default example.

* `headers(...)` Slice of response headers that are returned back to a caller.

* `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
  _`serde_json::json!`_ can parse as a _`serde_json::Value`_.

* `response = ...` Type what implements [`AsResponse`][as_response_trait] trait. This can alternatively be used to
   define response attributes. _`response`_ attribute cannot co-exist with other than _`status`_ attribute.

* `content((...), (...))` Can be used to define multiple return types for single response status. Supported format for single
  _content_ is `(content_type = response_body, example = "...", examples(...))`. _`example`_
  and _`examples`_ are optional arguments. Examples attribute behaves exactly same way as in
  the response and is mutually exclusive with the example attribute.

* `examples(...)` Define multiple examples for single response. This attribute is mutually
  exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
    * `name = ...` This is first attribute and value must be literal string.
    * `summary = ...` Short description of example. Value must be literal string.
    * `description = ...` Long description of example. Attribute supports markdown for rich text
      representation. Value must be literal string.
    * `value = ...` Example value. It must be _`json!(...)`_. _`json!(...)`_ should be something that
      _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
    * `external_value = ...` Define URI to literal example value. This is mutually exclusive to
      the _`value`_ attribute. Value must be literal string.

     _**Example of example definition.**_
    ```text
     ("John" = (summary = "This is John", value = json!({"name": "John"})))
    ```

**Minimal response format:**
```text
responses(
    (status = 200, description = "success response"),
    (status = 404, description = "resource missing"),
    (status = "5XX", description = "server error"),
    (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error"),
    (status = IM_A_TEAPOT, description = "happy easter")
)
```

**More complete Response:**
```text
responses(
    (status = 200, description = "Success response", body = Pet, content_type = "application/json",
        headers(...),
        example = json!({"id": 1, "name": "bob the cat"})
    )
)
```

**Response with multiple response content types:**
```text
responses(
    (status = 200, description = "Success response", body = Pet, content_type = ["application/json", "text/xml"])
)
```

**Multiple response return types with _`content(...)`_ attribute:**

_**Define multiple response return types for single response status with their own example.**_
```text
responses(
   (status = 200, content(
           ("application/vnd.user.v1+json" = User, example = json!(User {id: "id".to_string()})),
           ("application/vnd.user.v2+json" = User2, example = json!(User2 {id: 2}))
       )
   )
)
```

### Using `AsResponse` for reusable responses

_**`ReusableResponse` must be a type that implements [`AsResponse`][as_response_trait].**_
```text
responses(
    (status = 200, response = ReusableResponse)
)
```

_**[`AsResponse`][as_response_trait] can also be inlined to the responses map.**_
```text
responses(
    (status = 200, response = inline(ReusableResponse))
)
```

## Responses from `AsResponses`

_**Responses for a path can be specified with one or more types that implement
[`AsResponses`][as_responses_trait].**_
```text
responses(MyResponse)
```

# Response Header Attributes

* `name` Name of the header. E.g. _`x-csrf-token`_

* `type` Additional type of the header value. Can be `Type` or `inline(Type)`.
  The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
  With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
  [`AsSchema`][as_schema] types. **Reminder!** It's up to the user to use valid type for the
  response header.

* `description = "..."` Can be used to define optional description for the response header as str.

**Header supported formats:**

```text
("x-csrf-token"),
("x-csrf-token" = String, description = "New csrf token"),
```

# Params Attributes

The list of attributes inside the `parameters(...)` attribute can take two forms: [Tuples](#tuples) or [AsParameters
Type](#intoparams-type).

## Tuples

In the tuples format, parameters are specified using the following attributes inside a list of
tuples separated by commas:

* `name` _**Must be the first argument**_. Define the name for parameter.

* `parameter_type` Define possible type for the parameter. Can be `Type` or `inline(Type)`.
  The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
  With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
  [`AsSchema`][as_schema] types. Parameter type is placed after `name` with
  equals sign E.g. _`"id" = String`_

* `in` _**Must be placed after name or parameter_type**_. Define the place of the parameter.
  This must be one of the variants of [`parameter::ParameterIn`][in_enum].
  E.g. _`Path, Query, Header, Cookie`_

* `deprecated` Define whether the parameter is deprecated or not. Can optionally be defined
   with explicit `bool` value as _`deprecated = bool`_.

* `description = "..."` Define possible description for the parameter as str.

* `style = ...` Defines how parameters are serialized by [`ParameterStyle`][style]. Default values are based on _`in`_ attribute.

* `explode` Defines whether new _`parameter=value`_ is created for each parameter withing _`object`_ or _`array`_.

* `allow_reserved` Defines whether reserved characters _`:/?#[]@!$&'()*+,;=`_ is allowed within value.

* `example = ...` Can method reference or _`json!(...)`_. Given example
  will override any example in underlying parameter type.

##### Parameter type attributes

These attributes supported when _`parameter_type`_ is present. Either by manually providing one
or otherwise resolved e.g from path macro argument when _`yaml`_ crate feature is
enabled.

* `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
  an open value as a string. By default the format is derived from the type of the property
  according OpenApi spec.

* `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*

* `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*

* `nullable` Defines property is nullable (note this is different to non-required).

* `multiple_of = ...` Can be used to define multiplier for a value. Value is considered valid
  division will result an `integer`. Value must be strictly above _`0`_.

* `maximum = ...` Can be used to define inclusive upper bound to a `number` value.

* `minimum = ...` Can be used to define inclusive lower bound to a `number` value.

* `exclusive_maximum = ...` Can be used to define exclusive upper bound to a `number` value.

* `exclusive_minimum = ...` Can be used to define exclusive lower bound to a `number` value.

* `max_length = ...` Can be used to define maximum length for `string` types.

* `min_length = ...` Can be used to define minimum length for `string` types.

* `pattern = ...` Can be used to define valid regular expression in _ECMA-262_ dialect the field value must match.

* `max_items = ...` Can be used to define maximum items allowed for `array` fields. Value must
  be non-negative integer.

* `min_items = ...` Can be used to define minimum items allowed for `array` fields. Value must
  be non-negative integer.

**For example:**

```text
parameters(
    ("id" = String, Path, deprecated, description = "Pet database id"),
    ("name", Path, deprecated, description = "Pet name"),
    (
        "value" = inline(Option<[String]>),
        Query,
        description = "Value description",
        style = Form,
        allow_reserved,
        deprecated,
        explode,
        example = json!(["Value"])),
        max_length = 10,
        min_items = 1
    )
)
```

## AsParameters Type

In the AsParameters parameters format, the parameters are specified using an identifier for a type
that implements [`AsParameters`][as_parameters]. See [`AsParameters`][as_parameters] for an
example.

```text
parameters(MyParameters)
```

**Note!** that `MyParameters` can also be used in combination with the [tuples
representation](#tuples) or other structs.
```text
parameters(
    MyParameters1,
    MyParameters2,
    ("id" = String, Path, deprecated, description = "Pet database id"),
)
```


_**More minimal example with the defaults.**_
```
# use salvo_core::prelude::*;
# use salvo_oapi::AsSchema;
# #[derive(AsSchema, Extractible, serde::Deserialize, serde::Serialize, Debug)]
# struct Pet {
#    id: u64,
#    name: String,
# }
#
#[salvo_oapi::endpoint(
   request_body = Pet,
   responses(
        (status = 200, description = "Pet stored successfully", body = Pet,
            headers(
                ("x-cache-len", description = "Cache length")
            )
        ),
   ),
   parameters(
     ("x-csrf-token", Header, description = "Current csrf token of user"),
   )
)]
fn post_pet(res: &mut Response) {
    res.render(Json(Pet {
        id: 4,
        name: "bob the cat".to_string(),
    }));
}
```

_**Use of Rust's own `#[deprecated]` attribute will reflect to the generated OpenAPI spec and mark this operation as deprecated.**_
```
# use serde_json::json;
# use salvo_core::prelude::*;
# use salvo_oapi::{endpoint, extract::PathParam};
#[endpoint(
    responses(
        (status = 200, description = "Pet found from database")
    ),
    parameters(
        ("id", description = "Pet id"),
    )
)]
#[deprecated]
async fn get_pet_by_id(id: PathParam<i32>, res: &mut Response) {
    let json = json!({ "pet": format!("{:?}", id.value())});
    res.render(Json(json))
}
```

_**Example with multiple return types**_
```
# use salvo_core::prelude::*;
# trait User {}
# struct User1 {
#   id: String
# }
# impl User for User1 {}
#[salvo_oapi::endpoint(
    responses(
        (status = 200, content(
                ("application/vnd.user.v1+json" = User1, example = json!({"id": "id".to_string()})),
                ("application/vnd.user.v2+json" = User2, example = json!({"id": 2}))
            )
        )
    )
)]
async fn get_user() {
}
````

_**Example with multiple examples on single response.**_
``rust
# use salvo_core::prelude::*;

# #[derive(serde::Serialize, serde::Deserialize)]
# struct User {
#   name: String
# }
#[salvo_oapi::endpoint(
    responses(
        (status = 200, body = User,
            examples(
                ("Demo" = (summary = "This is summary", description = "Long description",
                            value = json!(User{name: "Demo".to_string()}))),
                ("John" = (summary = "Another user", value = json!({"name": "John"})))
             )
        )
    )
)]
async fn get_user() -> Json<User> {
  Json(User {name: "John".to_string()})
}
``

[in_enum]: salvo_oapi/openapi/path/enum.ParameterIn.html
[path]: trait.Path.html
[as_schema]: trait.AsSchema.html
[openapi]: derive.OpenApi.html
[security]: openapi/security/struct.SecurityRequirement.html
[security_schema]: openapi/security/struct.SecuritySchema.html
[primitive]: https://doc.rust-lang.org/std/primitive/index.html
[as_parameters]: trait.AsParameters.html
[style]: openapi/path/enum.ParameterStyle.html
[as_responses_trait]: trait.AsResponses.html
[as_parameters_derive]: derive.AsParameters.html
[as_response_trait]: trait.AsResponse.html
[known_format]: openapi/schema/enum.KnownFormat.html
[xml]: openapi/xml/struct.Xml.html