use crate::hue::error::HueError;
use crate::hue::models::{BridgeConnection, EntertainmentArea, EntertainmentChannel};
use openssl::ssl::{SslContext, SslContextBuilder, SslMethod, SslVerifyMode};
use tokio::net::UdpSocket;
use tokio_dtls_stream_sink::{Client, Session};
use tracing::{debug, trace};

pub struct EntertainmentStreamSession {
    _client: Client,
    session: Session,
    area_id: String,
    sequence_id: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct EntertainmentChannelColor {
    pub channel_id: u8,
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl EntertainmentStreamSession {
    pub async fn connect(
        connection: &BridgeConnection,
        area: &EntertainmentArea,
    ) -> Result<Self, HueError> {
        let client_key = connection
            .client_key
            .as_deref()
            .ok_or(HueError::MissingClientKey)?;
        let application_id =
            connection
                .application_id
                .as_deref()
                .ok_or(HueError::UnexpectedResponse(
                    "the bridge application id is missing for entertainment streaming",
                ))?;

        debug!(
            bridge_ip = %connection.bridge_ip,
            area_id = %area.id,
            application_id = ?connection.application_id,
            "connecting Hue Entertainment DTLS session"
        );
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|error| HueError::EntertainmentStream(error.to_string()))?;
        let client = Client::new(socket);
        let context = create_dtls_context(application_id, client_key)?;
        let session = client
            .connect((connection.bridge_ip.as_str(), 2100), Some(context))
            .await
            .map_err(|error| HueError::EntertainmentStream(error.to_string()))?;

        Ok(Self {
            _client: client,
            session,
            area_id: area.id.clone(),
            sequence_id: 0,
        })
    }

    pub async fn write_rgb_frame(
        &mut self,
        channel_colors: &[EntertainmentChannelColor],
    ) -> Result<(), HueError> {
        trace!(
            area_id = %self.area_id,
            sequence_id = self.sequence_id,
            channel_count = channel_colors.len(),
            "writing Hue entertainment frame"
        );
        let packet = encode_rgb_packet(&self.area_id, self.sequence_id, channel_colors)?;
        self.sequence_id = self.sequence_id.wrapping_add(1);
        self.session
            .write(&packet)
            .await
            .map_err(|error| HueError::EntertainmentStream(error.to_string()))
    }
}

pub fn empty_rgb_frame(area: &EntertainmentArea) -> Vec<EntertainmentChannelColor> {
    area.channels
        .iter()
        .map(|channel| EntertainmentChannelColor {
            channel_id: channel.channel_id,
            red: 0,
            green: 0,
            blue: 0,
        })
        .collect()
}

pub fn build_rgb_channels(
    channels: &[EntertainmentChannel],
    red: f32,
    green: f32,
    blue: f32,
    intensity: f32,
) -> Vec<EntertainmentChannelColor> {
    channels
        .iter()
        .map(|channel| {
            let x_bias = ((channel.position.x + 1.0) / 2.0).clamp(0.0, 1.0);
            let height_bias = ((channel.position.y + 1.0) / 2.0).clamp(0.0, 1.0);
            let spatial = (0.75 + 0.25 * height_bias).clamp(0.0, 1.0);
            let red_mix = red * (1.05 - 0.35 * x_bias);
            let green_mix = green * (0.9 + 0.2 * (1.0 - (x_bias - 0.5).abs() * 2.0));
            let blue_mix = blue * (0.7 + 0.45 * x_bias);
            let scale = (intensity * spatial).clamp(0.0, 1.0);

            EntertainmentChannelColor {
                channel_id: channel.channel_id,
                red: float_to_u16(red_mix * scale),
                green: float_to_u16(green_mix * scale),
                blue: float_to_u16(blue_mix * scale),
            }
        })
        .collect()
}

fn create_dtls_context(application_id: &str, client_key_hex: &str) -> Result<SslContext, HueError> {
    let psk = hex::decode(client_key_hex).map_err(|error| {
        HueError::EntertainmentStream(format!("invalid Hue client key: {error}"))
    })?;
    if psk.is_empty() {
        return Err(HueError::MissingClientKey);
    }

    let mut builder = SslContextBuilder::new(SslMethod::dtls_client())?;
    builder.set_min_proto_version(Some(openssl::ssl::SslVersion::DTLS1_2))?;
    builder.set_max_proto_version(Some(openssl::ssl::SslVersion::DTLS1_2))?;
    builder.set_cipher_list("PSK-AES128-GCM-SHA256")?;
    builder.set_verify(SslVerifyMode::NONE);

    let application_id = application_id.to_string();
    builder.set_psk_client_callback(move |_ssl, _hint, identity, psk_out| {
        if application_id.len() + 1 > identity.len() {
            return Err(openssl::error::ErrorStack::get());
        }
        identity[..application_id.len()].copy_from_slice(application_id.as_bytes());
        identity[application_id.len()] = 0;

        if psk.len() > psk_out.len() {
            return Err(openssl::error::ErrorStack::get());
        }

        psk_out[..psk.len()].copy_from_slice(&psk);
        Ok(psk.len())
    });

    Ok(builder.build())
}
fn encode_rgb_packet(
    area_id: &str,
    sequence_id: u8,
    channel_colors: &[EntertainmentChannelColor],
) -> Result<Vec<u8>, HueError> {
    let area_id = area_id.trim();
    if area_id.is_empty() {
        return Err(HueError::InvalidConfig("entertainment area ID is required"));
    }

    let mut packet = Vec::with_capacity(16 + 36 + channel_colors.len() * 7);
    packet.extend_from_slice(b"HueStream");
    packet.extend_from_slice(&[0x02, 0x00]);
    packet.push(sequence_id);
    packet.extend_from_slice(&[0x00, 0x00]);
    packet.push(0x00);
    packet.push(0x00);
    packet.extend_from_slice(area_id.as_bytes());

    for color in channel_colors.iter().take(20) {
        packet.push(color.channel_id);
        packet.extend_from_slice(&color.red.to_be_bytes());
        packet.extend_from_slice(&color.green.to_be_bytes());
        packet.extend_from_slice(&color.blue.to_be_bytes());
    }

    Ok(packet)
}

fn float_to_u16(value: f32) -> u16 {
    (value.clamp(0.0, 1.0) * u16::MAX as f32).round() as u16
}

#[cfg(test)]
mod tests {
    use super::{encode_rgb_packet, EntertainmentChannelColor};

    #[test]
    fn encodes_hue_stream_packet_header() {
        let packet = encode_rgb_packet(
            "1a8d99cc-967b-44f2-9202-43f976c0fa6b",
            7,
            &[EntertainmentChannelColor {
                channel_id: 0,
                red: u16::MAX,
                green: 0,
                blue: 0,
            }],
        )
        .unwrap();

        assert_eq!(&packet[..9], b"HueStream");
        assert_eq!(packet[9], 0x02);
        assert_eq!(packet[10], 0x00);
        assert_eq!(packet[11], 7);
    }
}
