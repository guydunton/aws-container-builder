#[cfg(test)]
mod test {

    use super::super::bootstrap::{ensure_working_dir_exists, get_amazon_linux_2_ami};
    use rusoto_core::request::HttpDispatchError;
    use rusoto_core::Region;
    use rusoto_ec2::Ec2Client;
    use rusoto_mock::*;
    use std::env::current_dir;
    use std::path::PathBuf;

    #[test]
    fn test_ensure_working_dir_exists_returns_ok_if_dir_exists() {
        let result = ensure_working_dir_exists(current_dir().unwrap(), |_| {
            Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists))
        });

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_ensure_working_dir_exists_returns_err_if_dir_cannot_be_created() {
        let result = ensure_working_dir_exists(PathBuf::from("./abc"), |_| {
            Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists))
        });

        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_ensure_returns_ok_if_dir_can_be_created() {
        let result = ensure_working_dir_exists(PathBuf::from("./abc"), |_| Ok(()));
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn linux_amis_should_return_newest_ami() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <DescribeImagesResult xmlns="http://ec2.amazonaws.com/doc/2016-11-15">
            <requestId>59dbff89-35bd-4eac-99ed-be587EXAMPLE</requestId> 
            <imagesSet>
                <item>
                    <creationDate>2019-10-31T06:01:09.000Z</creationDate>
                    <imageId>image1</imageId>
                </item>
                <item>
                    <creationDate>2019-10-31T06:01:10.000Z</creationDate>
                    <imageId>image2</imageId>
                </item>
            </imagesSet>
        </DescribeImagesResult>
        "#;

        let mock = MockRequestDispatcher::default().with_body(&response);
        let client = Ec2Client::new_with(mock, MockCredentialsProvider, Region::UsEast1);

        let result = get_amazon_linux_2_ami(&client).await;
        assert_eq!(result.unwrap(), "image2".to_owned());
    }

    #[tokio::test]
    async fn linux_amis_should_return_error_if_request_fails() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <DescribeImagesResult xmlns="http://ec2.amazonaws.com/doc/2016-11-15">
            <requestId>59dbff89-35bd-4eac-99ed-be587EXAMPLE</requestId> 
            <imagesSet>
                <item>
                    <creationDate>2019-10-31T06:01:09.000Z</creationDate>
                    <imageId>image1</imageId>
                </item>
                <item>
                    <creationDate>2019-10-31T06:01:10.000Z</creationDate>
                    <imageId>image2</imageId>
                </item>
            </imagesSet>
        </DescribeImagesResult>
        "#;

        let mock = MockRequestDispatcher::with_dispatch_error(HttpDispatchError::new(
            "synthetic error".to_owned(),
        ))
        .with_body(&response);

        let client = Ec2Client::new_with(mock, MockCredentialsProvider, Region::UsEast1);
        let result = get_amazon_linux_2_ami(&client).await;
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn linux_amis_errors_with_malformed_image() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <DescribeImagesResult xmlns="http://ec2.amazonaws.com/doc/2016-11-15">
            <requestId>59dbff89-35bd-4eac-99ed-be587EXAMPLE</requestId> 
        </DescribeImagesResult>
        "#;

        let mock = MockRequestDispatcher::default().with_body(&response);
        let client = Ec2Client::new_with(mock, MockCredentialsProvider, Region::UsEast1);

        let result = get_amazon_linux_2_ami(&client).await;
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn linux_amis_errors_with_missing_creation_dates() {
        let response = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <DescribeImagesResult xmlns="http://ec2.amazonaws.com/doc/2016-11-15">
            <requestId>59dbff89-35bd-4eac-99ed-be587EXAMPLE</requestId> 
            <imagesSet>
                <item>
                    <creationDate>2019-10-31T06:01:09.000Z</creationDate>
                </item>
            </imagesSet>
        </DescribeImagesResult>
        "#;

        let mock = MockRequestDispatcher::default().with_body(&response);
        let client = Ec2Client::new_with(mock, MockCredentialsProvider, Region::UsEast1);

        let result = get_amazon_linux_2_ami(&client).await;
        assert_eq!(result.is_err(), true);
    }
}
