use microslop_groups_extract::*;
use reqwest::{
    ClientBuilder,
    header::{self, HeaderMap, HeaderValue},
};
use std::{env, process};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: microslop_mail <account_id>");
        process::exit(1);
    }

    let account_id = args[1].clone();
    if account_id.len() != 36 {
        println!("The accound_id must 36 characters");
        process::exit(1);
    }

    println!("Account Id: {account_id}");

    let token = if let Ok(token) = env::var("ACCESS_TOKEN") {
        token
    } else {
        println!(
            "Provide the access token in the ACCESS_TOKEN env variable\nexport ACCESS_TOKEN=<access_token>"
        );
        process::exit(1);
    };

    let mut headers = HeaderMap::new();

    let mut auth_header_value = token.parse::<HeaderValue>().unwrap();
    auth_header_value.set_sensitive(true);

    headers.insert(header::AUTHORIZATION, auth_header_value);
    headers.insert(header::HOST, "graph.microsoft.com".parse().unwrap());
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
    headers.insert(
        header::REFERER,
        "https://myaccount.microsoft.com/".parse().unwrap(),
    );
    headers.insert(
        header::ORIGIN,
        "https://myaccount.microsoft.com/".parse().unwrap(),
    );

    let client_builder = ClientBuilder::new().default_headers(headers);
    let client = client_builder.build().unwrap();

    let groups = get_groups(account_id, client.clone()).await;
    let mut groups_id = Vec::new();
    for group in groups {
        println!(
            "╔═══════════════════════════════════════\n
            ║Querying users for group: {}\n
            ║Description: {}\n
            ║Id: {}\n
            ╚═══════════════════════════════════════",
            group.display_name, group.description, group.id
        );
        groups_id.push(group.id);
    }

    let first_url: Vec<[String; 2]> = groups_id.iter()
        .filter(|x| x.len() == 36)
        .map(|x| [x.to_string(), format!("https://graph.microsoft.com/beta/groups/{x}/transitiveMembers/microsoft.graph.user?$orderby=displayName+asc&$top=999&$select=id,userType,displayName,givenName,surname,userPrincipalName,mail,employeeType,department,createdDateTime,lastPasswordChangeDateTime")])
        .collect();
    //println!("{:?}", first_queries_url);

    let mut join_handles = Vec::new();

    for first_url in first_url {
        join_handles.push(tokio::spawn(save_users_from_group(
            first_url,
            client.clone(),
        )));
    }

    for handle in join_handles {
        handle.await.unwrap();
    }
}
