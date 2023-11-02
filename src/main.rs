use anyhow::{Ok, Result};
use aws_config::{meta::region::RegionProviderChain, retry::RetryConfig};
use aws_sdk_s3 as s3;
use aws_sdk_s3::{Client, Error};
use url::Url;

//creat a s3 client
pub async fn create_3_client(retries: u32) -> anyhow::Result<s3::Client> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let retry_config = RetryConfig::standard().with_max_attempts(retries);
    let config = aws_sdk_s3::config::Builder::from(&shared_config)
        .retry_config(retry_config)
        .build();
    let client = s3::Client::from_conf(config);

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get the directory path from command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <s3_uri, e.g.s3://bucket/prefix/> ", args[0]);
        return Ok(());
    }
    let s3_uri = &args[1];

    let url = Url::parse(s3_uri).unwrap();

    let bucket = url.host_str().unwrap();

    let prefix = url.path().trim_start_matches('/');

    let client = create_3_client(3).await?;
    let prefix_option = match prefix.is_empty() {
        true => None,
        false => Some(prefix.to_string()),
    };

    restore_deleted_objects_with_prefix(&client, bucket, &prefix_option).await?;
    check_deleted_objects_with_prefix(&client, bucket, &prefix_option).await?;

    Ok(())
}

async fn restore_deleted_objects_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &Option<String>,
) -> Result<()> {
    let mut next_key_marker: String = "".to_string();
    let mut first_run = true;
    loop {
        let output = match first_run {
            true => {
                first_run = false;
                client
                    .list_object_versions()
                    .bucket(bucket)
                    .set_prefix(prefix.clone())
                    .send()
                    .await?
            }
            false => {
                client
                    .list_object_versions()
                    .bucket(bucket)
                    .set_prefix(prefix.clone())
                    .key_marker(next_key_marker)
                    .send()
                    .await?
            }
        };

        for version in output.delete_markers() {
            for v in version {
                if v.is_latest() {
                    client
                        .delete_object()
                        .bucket(bucket)
                        .key(v.key().unwrap_or_default())
                        .version_id(v.version_id().unwrap_or_default())
                        .send()
                        .await?;
                    println!("Restored object: {}", v.key().unwrap_or_default());
                }
            }
        }

        if !output.is_truncated {
            break;
        }

        next_key_marker = output.next_key_marker().unwrap_or_default().to_string();
    }

    Ok(())
}

async fn check_deleted_objects_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &Option<String>,
) -> Result<()> {
    let mut next_key_marker: String = "".to_string();
    let mut first_run = true;
    loop {
        let output = match first_run {
            true => {
                first_run = false;
                client
                    .list_object_versions()
                    .bucket(bucket)
                    .set_prefix(prefix.clone())
                    .send()
                    .await?
            }
            false => {
                client
                    .list_object_versions()
                    .bucket(bucket)
                    .set_prefix(prefix.clone())
                    .key_marker(next_key_marker)
                    .send()
                    .await?
            }
        };

        for version in output.delete_markers() {
            for v in version {
                if v.is_latest() {
                    println!(
                        "Object which delete maker is still the lastest version: {}",
                        v.key().unwrap_or_default()
                    );
                }
            }
        }

        if !output.is_truncated {
            break;
        }

        next_key_marker = output.next_key_marker().unwrap_or_default().to_string();
    }

    Ok(())
}
