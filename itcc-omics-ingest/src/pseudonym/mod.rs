pub mod handler;

use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/*
#[derive(Debug, Serialize)]
struct CreateTokenReq<'a> {
    #[serde(rename = "type")]
    token_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "allowedUses")]
    allowed_uses: Option<u32>,
    data: TokenData<'a>,
}

#[derive(Debug, Serialize)]
struct TokenData<'a> {
    // Some Mainzelliste versions use idtypes vs idTypes.
    // We'll send "idtypes" here; if your server rejects it, change rename to "idTypes".
    #[serde(rename = "idtypes")]
    idtypes: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
struct CreateTokenResp {
    id: Uuid,
}

#[derive(Debug, Serialize)]
struct CreatePatientReq<'a> {
    // Depending on your Mainzelliste version:
    // - some use top-level patient fields
    // - some use { "fields": {...}, "ids": {...} }
    //
    // We'll use the "fields/ids" style (works for many ML API versions).
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    // Some servers accept "externalIds" or similar; keep generic if needed later
    extra: Option<Value>,
}

async fn main() -> Result<()> {
    let base_url = "http://localhost:7887";
    let api_key = "changeme1";
    let http = Client::new();

    // 1) Create session
    let session: CreateSessionResp = http
        .post(format!("{base_url}/sessions"))
        .header("mainzellisteApiKey", api_key)
        .send()
        .await
        .context("POST /sessions failed")?
        .error_for_status()
        .context("POST /sessions returned error")?
        .json()
        .await
        .context("parse /sessions JSON failed")?;

    println!("sessionId = {}", session.session_id);

    // 2) Create addPatient token requesting pid/localid/cryptoid
    let token_req = CreateTokenReq {
        token_type: "addPatient",
        allowed_uses: Some(1),
        data: TokenData {
            idtypes: vec!["pid", "localid", "cryptoid"],
        },
    };

    let token: CreateTokenResp = http
        .post(format!("{base_url}/sessions/{}/tokens", session.session_id))
        .header("mainzellisteApiKey", api_key)
        .json(&token_req)
        .send()
        .await
        .context("POST /sessions/{sessionId}/tokens failed")?
        .error_for_status()
        .context("token endpoint returned error")?
        .json()
        .await
        .context("parse token JSON failed")?;

    println!("tokenId = {}", token.id);

    // 3) Create patient
    //
    // A) If you have identifying fields:
    let fields = serde_json::json!({
        "first_name": "Andreas",
        "surname": "Groß",
        "birth_year": 1989,
        "birth_month": 11,
        "birth_day": 9,
        "zip": 12355,
        "city": "Berlin"
    });

    // B) If you also have an external ID type configured (example: extid):
    // let ids = serde_json::json!({ "extid": "PATIENT-12345" });

    let patient_req = CreatePatientReq {
        fields: Some(fields),
        ids: None, // change to Some(ids) if you configured an external id type
        extra: None,
    };

    // Try tokenId and tokenid because ML variants differ.
    let mut resp = http
        .post(format!("{base_url}/patients?tokenId={}", token.id))
        .header("mainzellisteApiKey", api_key)
        .json(&patient_req)
        .send()
        .await
        .context("POST /patients failed")?;

    if resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST {
        // fallback to tokenid
        resp = http
            .post(format!("{base_url}/patients?tokenid={}", token.id))
            .header("mainzellisteApiKey", api_key)
            .json(&patient_req)
            .send()
            .await
            .context("POST /patients (tokenid fallback) failed")?;
    }

    if resp.status() != StatusCode::CREATED {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("create patient failed: {status} body={body}"));
    }

    // Response structure differs by version; parse as JSON and extract IDs.
    let body: Value = resp.json().await.context("parse /patients JSON failed")?;
    println!("create patient response:\n{}", serde_json::to_string_pretty(&body)?);

    // Try common shapes:
    // - {"ids":{"pid":"...","localid":"...","cryptoid":"..."}}
    // - or list-of-ids objects
    let pid = extract_id(&body, "pid");
    let localid = extract_id(&body, "localid");
    let cryptoid = extract_id(&body, "cryptoid");

    println!("pid     = {:?}", pid);
    println!("localid = {:?}", localid);
    println!("cryptoid= {:?}", cryptoid);

    Ok(())
}

// Extract id in a few common Mainzelliste response shapes.
fn extract_id(v: &Value, id_type: &str) -> Option<String> {
    // Shape 1: { "ids": { "pid": "..." } }
    if let Some(s) = v.pointer(&format!("/ids/{id_type}")).and_then(|x| x.as_str()) {
        return Some(s.to_string());
    }

    // Shape 2: { "ids": [ { "idType":"pid","idString":"..." }, ... ] }
    if let Some(ids) = v.get("ids").and_then(|x| x.as_array()) {
        for item in ids {
            let t = item.get("idType").and_then(|x| x.as_str());
            let s = item.get("idString").and_then(|x| x.as_str());
            if t == Some(id_type) {
                return s.map(|x| x.to_string());
            }
        }
    }

    // Shape 3: nested arrays (readPatients often returns [[{...}]] ):
    if let Some(arr) = v.as_array() {
        for el in arr {
            if let Some(found) = extract_id(el, id_type) {
                return Some(found);
            }
        }
    }

    // Shape 4: recurse through objects shallowly
    if let Some(obj) = v.as_object() {
        for (_k, val) in obj {
            if let Some(found) = extract_id(val, id_type) {
                return Some(found);
            }
        }
    }

    None
}


 */