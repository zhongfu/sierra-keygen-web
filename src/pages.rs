use markup;
use strum::IntoEnumIterator;

markup::define! {
    Main(
        device_generation: Option<sierra_keygen::DeviceGeneration>,
        challenge_type: Option<sierra_keygen::ChallengeType>,
        challenge: Option<String>,
        challenge_response: Option<Vec<u8>>,
        hcaptcha_sitekey: Option<String>,
        error_msg: Option<String>
    ) {
        html {
            head {
                title { "sierra-keygen" }
                @if let Some(_) = hcaptcha_sitekey {
                        script[src = "https://js.hcaptcha.com/1/api.js?recaptchacompat=off", async, defer] {}
                }
            }
            body {
                h1 { "sierra-keygen" }
                @if let Some(err_msg) = error_msg {
                    p[class = "error", style = "color: red; weight: bold;"] {
                        @err_msg
                    }
                }
                form[action = Some("/"), method = Some("POST")] {
                    table {
                        tbody {
                            tr {
                                td {
                                    label[for = "device_generation"] {
                                        "Device generation"
                                    }
                                }
                                td {
                                    select[name = "device_generation", id = "device_generation"] {
                                        option {}
                                        @for g in sierra_keygen::DeviceGeneration::iter() {
                                            @if device_generation.as_ref().map(|f| f.eq(&g)).unwrap_or(false) {
                                                option[value = g.to_string(), selected] {
                                                    @g.to_string()
                                                }
                                            } else {
                                                option[value = g.to_string()] {
                                                    @g.to_string()
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            tr {
                                td {
                                    label[for = "challenge_type"] {
                                        "Challenge type"
                                    }
                                }
                                td {
                                    select[name = "challenge_type", id = "challenge_type"] {
                                        option {}
                                        @for t in sierra_keygen::ChallengeType::iter() {
                                            @if challenge_type.as_ref().map(|f| f.eq(&t)).unwrap_or(false) {
                                                option[value = t.to_string(), selected] {
                                                    @t.to_string()
                                                }
                                            } else {
                                                option[value = t.to_string()] {
                                                    @t.to_string()
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            tr {
                                td {
                                    label[for = "challenge"] {
                                        "Challenge"
                                    }
                                }
                                td {
                                    @if let Some(challenge) = challenge {
                                        input[type = "text", name = "challenge", id = "challenge", value = challenge] {}
                                    } else {
                                        input[type = "text", name = "challenge", id = "challenge"] {}
                                    }
                                }

                            }

                            @if let Some(hcaptcha_site_id) = hcaptcha_sitekey {
                                tr {
                                    td {}
                                    td {
                                        div[class = "h-captcha", "data-sitekey" = hcaptcha_site_id] {}
                                    }
                                }
                            }

                            tr {
                                td {}
                                td {
                                    input[type = "submit", value = "Generate"] {}
                                }
                            }

                            @if let Some(challenge_type) = challenge_type {
                                @if let Some(challenge_response) = challenge_response {
                                    // spacer
                                    tr[style = "height: 20px;"] {}
                                    tr {
                                        td {
                                            "Received challenge:"
                                        }
                                        td {
                                            p[style = "font-family: monospace;"] {
                                                @format!("{}", challenge.as_ref().unwrap().to_uppercase())
                                            }
                                        }
                                    }
                                    tr {
                                        td {
                                            "Challenge response:"
                                        }
                                        td {
                                            p[style = "font-family: monospace;"] {
                                                @format!("AT!{}=\"{}\"", challenge_type.to_string(), hex::encode(challenge_response).to_uppercase())
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

}
