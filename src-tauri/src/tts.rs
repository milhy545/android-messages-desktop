// No unused imports
pub async fn speak(text: &str) -> Result<(), String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Error as WsError};
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    let url = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";
    let request = url.into_client_request().map_err(|e| e.to_string())?;

    let (ws_stream, _) = connect_async(request).await.map_err(|e: WsError| e.to_string())?;
    let (mut write, mut read) = ws_stream.split();

    let uuid = uuid::Uuid::new_v4().to_string().replace("-", "");

    let config_msg = format!(
        "X-Timestamp: {}\r\nContent-Type: application/json; charset=utf-8\r\nPath: speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":false,\"wordBoundaryEnabled\":false}},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}",
        chrono::Utc::now().format("%a %b %d %Y %H:%M:%S GMT+0000 (Coordinated Universal Time)")
    );

    write.send(Message::Text(config_msg.into())).await.map_err(|e: WsError| e.to_string())?;

    let ssml = format!(
        "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='cs-CZ'><voice name='cs-CZ-AntoninNeural'><prosody pitch='+0Hz' rate='0%' volume='100%'>{}</prosody></voice></speak>",
        text
    );

    let ssml_msg = format!(
        "X-RequestId: {}\r\nContent-Type: application/ssml+xml\r\nPath: ssml\r\n\r\n{}",
        uuid, ssml
    );

    write.send(Message::Text(ssml_msg.into())).await.map_err(|e: WsError| e.to_string())?;

    let mut audio_data = Vec::new();

    while let Some(msg_result) = read.next().await {
        let msg = msg_result.map_err(|e: WsError| e.to_string())?;
        match msg {
            Message::Binary(bin) => {
                if let Some(pos) = bin.windows(4).position(|window| window == b"\r\n\r\n") {
                    let data_start = pos + 4;
                    audio_data.extend_from_slice(&bin[data_start..]);
                }
            },
            Message::Text(text) => {
                if text.contains("Path: turn.end") {
                    break;
                }
            },
            _ => {}
        }
    }

    if audio_data.is_empty() {
        return Err("No audio data received".to_string());
    }

    std::thread::spawn(move || {
        let path = std::env::temp_dir().join("tts_output.mp3");
        std::fs::write(&path, &audio_data).unwrap();
        println!("Saved TTS audio to {:?}", path);
    });

    Ok(())
}
