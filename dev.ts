import { faker, fakerDE } from '@faker-js/faker'
// or, if desiring a different locale
// import { fakerDE as faker } from '@faker-js/faker'

console.log(faker.internet.ip())
console.log(fakerDE.internet.ip())

console.log(faker.person.fullName())
console.log(fakerDE.person.fullName())

console.log(faker.internet.email())
console.log(fakerDE.internet.email())
