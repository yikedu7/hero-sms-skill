# HeroSMS API coverage

Generated from `docs/openapi.json`. **Outbound client APIs** are invoked via the Rust `herosms` CLI.
**Receiving OTP** in agent environments defaults to **polling** (`wait-otp`, `getStatusV2`, etc.), not webhooks.

## Outbound client API (50 operations)

| Covered | Method | Path | operationId | Notes |
| --- | --- | --- | --- | --- |

### Activations

| yes | GET | `/activations` | `getActiveActivations` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/activations` | `buyActivationBatch` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/history` | `82eacb597b010298c78dacd1f0bdd3ff` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/offers/{verificationType}` | `getActivationOffers` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/stats` | `getActivationStats` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | DELETE | `/activations/{activationId}` | `cancelActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/activations/{activationId}/finish` | `finishActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/activations/{activationId}/prolong` | `prolongActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/{activationId}/prolong/history` | `prolongActivationHistory` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/{activationId}/prolong/options` | `prolongOptions` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/activations/{activationId}/reactivate` | `reactivateActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/{activationId}/reactivate/options` | `reactivateOptions` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/activations/{activationId}/replace` | `replaceActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

### Activation OTP

| yes | GET | `/activations/{activationId}/otp` | `getActivationOtpList` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/activations/{activationId}/otp/last` | `getActivationOtpLast` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

### Emails

| yes | GET | `/emails` | `7f17119e77aad0c154d2f5e600ef092a` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/emails` | `22164e5067a27560b8fbf28b2ac1f019` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/emails/batch` | `7892a59d83e6e38d154854fe1f1b065f` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/emails/domains` | `59f271906bdbc81b6b2cdd4213b01775` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/emails/offers` | `f8e545c04e941382450fcc88b14c4e89` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | DELETE | `/emails/{emailId}` | `20dae8daa691db1282f0d1d268dbac44` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `/emails/{emailId}` | `c57ebd15edbfa8625b8a60984b76e3c6` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `/emails/{emailId}/reorder` | `bb0d53e0557982f130c1540ed663e831` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

### SMS-Activate

| yes | GET | `?action=cancelActivation` | `saCancelActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=finishActivation` | `saFinishActivation` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getActiveActivations` | `saGetActiveActivations` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getAllSms` | `getAllSms` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getBalance` | `getBalance` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getCountries` | `getCountries` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getHistory` | `getHistory` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getNumber` | `getNumber` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getNumberV2` | `getNumberV2` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getOperators` | `getOperators` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getPrices` | `getPrices` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getRentNumber` | `getRentNumber` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getRentServicesAndCountries` | `getRentServicesAndCountries` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getServicesList` | `getServicesList` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getStatus` | `getStatus` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getStatusV2` | `getStatusV2` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getTopCountriesByService` | `getTopCountriesByService` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=getTopCountriesByServiceRank` | `getTopCountriesByServiceRank` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `?action=prolong` | `prolong` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=prolongHistory` | `prolongHistory` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=prolongOptions` | `saProlongOptions` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | POST | `?action=reactivate` | `reactivate` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=reactivateOptions` | `saReactivateOptions` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=serviceCountRent` | `serviceCountRent` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |
| yes | GET | `?action=setStatus` | `setStatus` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

### Classifiers

| yes | GET | `/classifiers/activations/custom-durations` | `customDurations` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

### Statistics

| yes | GET | `/stats/deliverability` | `60679e939ef779f9063f3a035599e820` | Outbound — `herosms call <operationId>` (receive OTP via `wait-otp` / polling, not required for coverage) |

## Webhooks

Webhooks are **platform → your HTTPS endpoint**. Grok Bot / typical cloud agents **do not** host inbound HTTPS; mark **receive** as **N/A** there and use polling.

### Platform pushes (HeroSMS → URL you configure)

| Covered | Direction | operationId | We receive in bot? | Notes |
| --- | --- | --- | --- | --- |
| yes | Platform → customer URL | `webhook-url-registration` | N/A | Configure up to **3 HTTPS URLs** in account personal information. No client API. |
| yes | Platform → customer URL | `smsIncoming` | **N/A** | POST JSON per `webhooks.sms-incoming`; documented in `skill/SKILL.md`. Requires operator-owned public HTTPS. |

### Skill / operator receive (optional, not a bot dependency)

| Covered | Component | Bot environment | Notes |
| --- | --- | --- | --- |
| partial | `herosms wait-otp` | **yes** (polling) | **Default** receive path; uses `getStatusV2` / `getStatus` / REST OTP. |
| partial | `herosms-webhook` sample | **N/A** | Optional reference server if operator runs TLS + public URL; not required for skill delivery. |
| partial | `herosms::webhook` parse helpers | **N/A** | For custom receivers / optional forward path only. |

## Summary

- **Outbound API operations:** 50 / 50 via `herosms call`
- **OTP receive (agents):** polling (`herosms wait-otp`) — covered
- **Webhook receive (agents):** N/A — documented only
- **CLI:** `cargo build -p herosms --release`
