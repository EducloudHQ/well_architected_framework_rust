use async_trait::async_trait;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::Client as S3Client;
use tracing::info;

#[async_trait]
pub trait GetFile {
    async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError>;
}

#[async_trait]
impl GetFile for S3Client {
    async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError> {
        info!("get file bucket {}, key {}", bucket, key);

        let output = self.get_object().bucket(bucket).key(key).send().await;

        return match output {
            Ok(response) => {
                let bytes = response.body.collect().await.unwrap().to_vec();
                info!("Object is downloaded, size is {}", bytes.len());
                Ok(bytes)
            }
            Err(err) => {
                let service_err = err.into_service_error();
                let meta = service_err.meta();
                info!("Error from aws when downloding: {}", meta.to_string());
                Err(service_err)
            }
        };
    }
}
