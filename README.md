# Autumn

## Description

Autumn is the microservice responsible for storing files and attachments.

**Features:**

- Save files locally or on S3.
- Support for different tags / buckets with different file requirements.
- Strips metadata from JPEGs and video files.

## Stack

- [Actix-Web](https://actix.rs/)
- [rust-s3](https://github.com/durch/rust-s3)
- [MongoDB](https://mongodb.com/)

## Resources

### Revolt

- [Revolt Project Board](https://github.com/revoltchat/revolt/discussions) (Submit feature requests here)
- [Revolt Testers Server](https://app.revolt.chat/invite/Testers)
- [Contribution Guide](https://developers.revolt.chat/contributing)

## CLI Commands

| Command            | Description                                                                                |
| ------------------ | ------------------------------------------------------------------------------------------ |
| `cargo build`      | Build/compile Autumn.                                                                      |
| `cargo run`        | Run Autumn.                                                                                |
| `cargo fmt`        | Format Autumn. Not intended for PR use to avoid accidentally formatting unformatted files. |

## Contributing

The contribution guide is located at [developers.revolt.chat/contributing](https://developers.revolt.chat/contributing).
Please note that a pull request should only take care of one issue so that we can review it quickly.

## License

Autumn is licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/autumn/blob/master/LICENSE).

## To-Do

- Make EXIF stripping optional, but on by default. (?exif=false)
