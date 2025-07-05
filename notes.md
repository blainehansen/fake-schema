For relative references inside objects, the context we want to look at is the context of the object we're generating, not of the entire thing, because we want to skip going all the way up the path to get to where we already are

Maybe have total generated and relative generated?



https://clickhouse.com/docs/getting-started/example-datasets




The core of

- composable

```
#Geo = [
  ({ country_code: USA, central_lat_long: (33.8475, -170.5953) }, 0.10),
  ({ country_code: GBR, central_lat_long: (55.5555, 55.5555) }, 0.10),
  // etc
]

company = [
  { name: "Rebel Alliance", id: 0000, domain: "rebels.org" },
  ...{
    name: company.name,
    id: string.uuid,
    ...helpers.weightedArrayElement(#Geo),
    domain: internet.domainName,
  },
]

users = [
  { first_name: "Leia", last_name: "Organa", company_id: 0000 },
  ...helpers.
  ...{
    first_name: person.firstname,
    middle_name: helpers.maybe(person.middlename, 0.20),
    last_name: helpers.maybe(person.lastname, 0.80),
    full_name: WordConcat(first_name, middle_name, last_name),
    company_id: ref(company)/id,
    email: format("{first_name}.{last_name}@{lookup(company_id)/company_domain}"),
  },
]
```
