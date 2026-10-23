# HeroSMS API coverage

Checklist generated from `docs/openapi.json` (official HeroSMS OpenAPI 3.2).

| Covered | Method | Path | operationId | Notes |
| --- | --- | --- | --- | --- |

## Activations

| yes | GET | `/activations` | `getActiveActivations` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/activations` | `buyActivationBatch` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/history` | `82eacb597b010298c78dacd1f0bdd3ff` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/offers/{verificationType}` | `getActivationOffers` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/stats` | `getActivationStats` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | DELETE | `/activations/{activationId}` | `cancelActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/activations/{activationId}/finish` | `finishActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/activations/{activationId}/prolong` | `prolongActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/{activationId}/prolong/history` | `prolongActivationHistory` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/{activationId}/prolong/options` | `prolongOptions` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/activations/{activationId}/reactivate` | `reactivateActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/{activationId}/reactivate/options` | `reactivateOptions` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/activations/{activationId}/replace` | `replaceActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## Activation OTP

| yes | GET | `/activations/{activationId}/otp` | `getActivationOtpList` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/activations/{activationId}/otp/last` | `getActivationOtpLast` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## Emails

| yes | GET | `/emails` | `7f17119e77aad0c154d2f5e600ef092a` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/emails` | `22164e5067a27560b8fbf28b2ac1f019` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/emails/batch` | `7892a59d83e6e38d154854fe1f1b065f` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/emails/domains` | `59f271906bdbc81b6b2cdd4213b01775` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/emails/offers` | `f8e545c04e941382450fcc88b14c4e89` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | DELETE | `/emails/{emailId}` | `20dae8daa691db1282f0d1d268dbac44` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `/emails/{emailId}` | `c57ebd15edbfa8625b8a60984b76e3c6` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `/emails/{emailId}/reorder` | `bb0d53e0557982f130c1540ed663e831` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## SMS-Activate

| yes | GET | `?action=cancelActivation` | `saCancelActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=finishActivation` | `saFinishActivation` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getActiveActivations` | `saGetActiveActivations` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getAllSms` | `getAllSms` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getBalance` | `getBalance` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getCountries` | `getCountries` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getHistory` | `getHistory` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getNumber` | `getNumber` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getNumberV2` | `getNumberV2` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getOperators` | `getOperators` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getPrices` | `getPrices` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getRentNumber` | `getRentNumber` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getRentServicesAndCountries` | `getRentServicesAndCountries` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getServicesList` | `getServicesList` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getStatus` | `getStatus` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getStatusV2` | `getStatusV2` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getTopCountriesByService` | `getTopCountriesByService` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=getTopCountriesByServiceRank` | `getTopCountriesByServiceRank` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `?action=prolong` | `prolong` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=prolongHistory` | `prolongHistory` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=prolongOptions` | `saProlongOptions` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | POST | `?action=reactivate` | `reactivate` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=reactivateOptions` | `saReactivateOptions` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=serviceCountRent` | `serviceCountRent` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |
| yes | GET | `?action=setStatus` | `setStatus` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## Classifiers

| yes | GET | `/classifiers/activations/custom-durations` | `customDurations` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## Statistics

| yes | GET | `/stats/deliverability` | `60679e939ef779f9063f3a035599e820` | Outbound REST/SMS-Activate — generic CLI `herosms call <operationId>` |

## Webhooks

HeroSMS delivers webhooks to **your** HTTPS endpoints. There is **no outbound API** in the spec to register URLs.

### Platform-side (account configuration)

| Covered | Side | operationId | Notes |
| --- | --- | --- | --- |
| yes | **Platform** | `webhook-url-registration` | Configure up to **3 HTTPS URLs** in HeroSMS account personal information (see OpenAPI `tags.Webhooks`). Not callable via `herosms` CLI. |

### Inbound events (HeroSMS → your server)

| Covered | Method | Path | operationId | Notes |
| --- | --- | --- | --- | --- |
| yes | POST | `webhook:sms-incoming` | `smsIncoming` | Inbound POST JSON per OpenAPI `webhooks.sms-incoming`; see `skill/SKILL.md` (Webhooks) and `herosms-webhook` receiver. |

### Skill-side (this repository)

| Covered | Component | Notes |
| --- | --- | --- |
| yes | `herosms::webhook` | Parses/validates `smsIncoming` payload; OTP extraction helper. |
| yes | `herosms-webhook` binary | Runnable sample HTTP server: `cargo run -p herosms-webhook --release`. TLS via reverse proxy. |

## Summary

- **Total tracked items:** 52 (50 outbound API operations + 1 platform webhook config + 1 inbound webhook event + skill receiver/libs)
- **Outbound CLI:** `cargo build -p herosms --release` → `target/release/herosms`
- **Webhook receiver:** `cargo run -p herosms-webhook --release`
- **Known gaps:** OpenAPI does not define HMAC/signature headers for webhooks; optional shared-secret header supported in sample receiver only.
