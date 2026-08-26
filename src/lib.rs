use reqwest::{
    Client, ClientBuilder,
    header::{self, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::fs;

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
    let mut iteration: u32 = 0;

    while !next_link.is_empty() {
        iteration += 1;
        println!("Group: {}, Page: {iteration}", group[0]);

        let response_bytes = client
            .get(&next_link)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        #[cfg(debug_assertions)]
        {
            let _ = fs::write(format!("{}_{iteration}.txt", group[0]), &response_bytes);
        }

        let mut parsed_response: ListUsersResponse =
            serde_json::from_slice(&response_bytes).unwrap();

        next_link.clear();
        users.append(&mut parsed_response.value);
        next_link = parsed_response.next_link;

        //std::thread::sleep(std::time::Duration::new(0, 500_000_000));
    }

    // Add lines to csv
    let mut writter = csv::Writer::from_path(format!("data/{}.csv", group[0])).unwrap();
    for user in &users {
        writter.serialize(user).unwrap();
    }
    // Add lines to json
    fs::write(
        format!("data/{}.json", group[0]),
        serde_json::to_string_pretty(&users).unwrap(),
    )
    .unwrap();
}

pub fn new_client_from_token(token: &String, host: &String) -> Client {
    let mut headers = HeaderMap::new();

    let mut auth_header_value = token.parse::<HeaderValue>().unwrap();
    auth_header_value.set_sensitive(true);

    headers.insert(header::AUTHORIZATION, auth_header_value);
    headers.insert(header::HOST, host.parse().unwrap());
    headers.insert(
        header::USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:154.0) Gecko/20100101 Firefox/154.0"
            .parse()
            .unwrap(),
    );
    headers.insert(header::ACCEPT, "*/*".parse().unwrap());
    headers.insert(
        header::ACCEPT_LANGUAGE,
        "fr,en-US;q=0.9,en;q=0.8".parse().unwrap(),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        "gzip, deflate, br, zstd".parse().unwrap(),
    );

    let client_builder = ClientBuilder::new().default_headers(headers);
    client_builder.build().unwrap()
}
