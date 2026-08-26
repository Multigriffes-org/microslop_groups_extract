use microslop_groups_extract::*;
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

    let host = "graph.microsoft.com".to_string();
    let Ok(token) = env::var("ACCESS_TOKEN") else {
        println!(
            "Provide the access token in the ACCESS_TOKEN env variable\nexport ACCESS_TOKEN=<access_token>"
        );
        process::exit(1);
    };
    //let token = if let Ok(token) = env::var("ACCESS_TOKEN") {
    //    token
    //} else {
    //    println!(
    //        "Provide the access token in the ACCESS_TOKEN env variable\nexport ACCESS_TOKEN=<access_token>"
    //    );
    //    process::exit(1);
    //};

    let client = new_client_from_token(&token, &host);

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

    let firsts_url: Vec<[String; 2]> = groups_id.iter()
        .filter(|x| x.len() == 36)
        .map(|x| [x.to_string(), format!("https://graph.microsoft.com/beta/groups/{x}/transitiveMembers/microsoft.graph.user?$orderby=displayName+asc&$top=999&$select=id,userType,displayName,givenName,surname,userPrincipalName,mail,employeeType,department,createdDateTime,lastPasswordChangeDateTime")])
        .collect();
    //println!("{:?}", first_queries_url);

    let mut join_handles = Vec::new();
    for first_url in firsts_url {
        join_handles.push(tokio::spawn(save_users_from_group(
            first_url,
            client.clone(),
        )));
    }

    for handle in join_handles {
        handle.await.unwrap();
    }
}
