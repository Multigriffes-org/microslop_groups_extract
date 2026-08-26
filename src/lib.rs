use std::fs;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ListGroupsResponse {
    value: Vec<Group>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
struct ListUsersResponse {
    #[serde(rename(deserialize = "@odata.nextLink"))]
    #[serde(default = "String::new")]
    next_link: String,
    value: Vec<User>,
}

#[derive(Serialize, Deserialize)]
enum UserType {
    Member,
    Guest,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    user_type: Option<UserType>,
    display_name: String,
    given_name: Option<String>,
    surname: Option<String>,
    user_principal_name: String,
    mail: Option<String>,
    employee_type: Option<String>,
    department: Option<String>,
    created_date_time: String,
    last_password_change_date_time: Option<String>,
}

pub async fn get_groups(account_id: String, client: Client) -> Vec<Group> {
    let url = format!(
        "https://graph.microsoft.com/v1.0/users/{account_id}/transitiveMemberOf/microsoft.graph.group?$orderby=displayName+asc&$top=999"
    );
    client
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<ListGroupsResponse>()
        .await
        .unwrap()
        .value
}

pub async fn save_users_from_group(group: [String; 2], client: Client) {
    match fs::read_dir("data/") {
        Ok(_) => {}
        Err(_) => {
            let _ = fs::create_dir("data/");
        }
    };

    let mut users: Vec<User> = Vec::new();
    let mut next_link = group[1].clone();
    let mut writter = csv::Writer::from_path(format!("data/{}.csv", group[0])).unwrap();
    let mut iteration: u32 = 0;

    while !next_link.is_empty() {
        iteration += 1;
        println!("Group: {}, Page: {iteration}", group[0]);
        //let mut response = client
        //    .get(&next_link)
        //    .send()
        //    .await
        //    .unwrap()
        //    .bytes()
        //    .await
        //    .unwrap();
        //
        //let _ = std::fs::write("test.txt", &response);
        //next_link = String::new();

        let mut response: ListUsersResponse = client
            .get(&next_link)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        // Add lines to csv
        for user in &response.value {
            writter.serialize(user).unwrap();
        }

        users.append(&mut response.value);
        next_link = response.next_link;

        //std::thread::sleep(std::time::Duration::new(0, 500_000_000));
    }

    // Add lines to json
    fs::write(
        format!("data/{}.json", group[0]),
        serde_json::to_string_pretty(&users).unwrap(),
    )
    .unwrap();
}
