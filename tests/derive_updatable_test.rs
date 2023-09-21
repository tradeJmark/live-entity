use fullstack_entity::derive::Updatable;
use fullstack_entity::Updatable;

#[derive(Updatable)]
struct Person {
    first_name: String,
    last_name: String
}

#[test]
fn test_derived_updatable() {
    let mut person = Person { first_name: "Paul".to_owned(), last_name: "Atreides".to_owned() };
    let update = UpdatedPerson::default().first_name("Leto".to_owned());
    person.update(&update);
    assert_eq!("Leto", person.first_name);
    assert_eq!("Atreides", person.last_name);
}