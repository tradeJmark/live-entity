use fullstack_entity_derive::Entity;
use serde::{Serialize, Deserialize};
use fullstack_entity::{Entity, Updatable};

#[derive(Debug, Serialize, Deserialize, Clone, Entity)]
struct Article {
    #[entity_id]
    id: i32,
    title: String,
    body: String
}

#[test]
fn test_derived_entity() {
    let id = 1;
    let title = "An examination of the consequences of bothering to do stuff".to_owned();
    let body = "Ah, screw it, I don't feel like it.".to_owned();
    let mut article = Article {
        id,
        title: title.clone(),
        body: body.clone()
    };

    assert_eq!(&id, article.get_id());

    let same_by_eq = Article {
        id,
        title: "An examination of the futility of searching for the perfect title".to_owned(),
        body: "I mean, it's just not worth the time.".to_owned()
    };
    assert_eq!(article, same_by_eq);

    let new_title = "There are no perfect titles".to_owned();
    let update = UpdatedArticle::default().title(new_title.clone());
    article.update(&update);
    assert_eq!(new_title, article.title);
    assert_eq!(body, article.body)
}