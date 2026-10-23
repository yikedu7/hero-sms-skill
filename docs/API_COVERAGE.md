# HeroSMS API coverage

Checklist generated from `docs/openapi.json` (official HeroSMS OpenAPI 3.2).
Each operation is covered by the generic OpenAPI-driven CLI unless noted.

| Covered | Method | Path | operationId | Notes |
| --- | --- | --- | --- | --- |

## Activations

| yes | GET | `/activations` | `getActiveActivations` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/activations` | `buyActivationBatch` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/history` | `82eacb597b010298c78dacd1f0bdd3ff` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/offers/{verificationType}` | `getActivationOffers` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/stats` | `getActivationStats` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | DELETE | `/activations/{activationId}` | `cancelActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/activations/{activationId}/finish` | `finishActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/activations/{activationId}/prolong` | `prolongActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/{activationId}/prolong/history` | `prolongActivationHistory` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/{activationId}/prolong/options` | `prolongOptions` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/activations/{activationId}/reactivate` | `reactivateActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/{activationId}/reactivate/options` | `reactivateOptions` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/activations/{activationId}/replace` | `replaceActivation` | Generic CLI `call` via skill/scripts/herosms.py |

## Activation OTP

| yes | GET | `/activations/{activationId}/otp` | `getActivationOtpList` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/activations/{activationId}/otp/last` | `getActivationOtpLast` | Generic CLI `call` via skill/scripts/herosms.py |

## Emails

| yes | GET | `/emails` | `7f17119e77aad0c154d2f5e600ef092a` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/emails` | `22164e5067a27560b8fbf28b2ac1f019` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/emails/batch` | `7892a59d83e6e38d154854fe1f1b065f` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/emails/domains` | `59f271906bdbc81b6b2cdd4213b01775` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/emails/offers` | `f8e545c04e941382450fcc88b14c4e89` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | DELETE | `/emails/{emailId}` | `20dae8daa691db1282f0d1d268dbac44` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `/emails/{emailId}` | `c57ebd15edbfa8625b8a60984b76e3c6` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `/emails/{emailId}/reorder` | `bb0d53e0557982f130c1540ed663e831` | Generic CLI `call` via skill/scripts/herosms.py |

## SMS-Activate

| yes | GET | `?action=cancelActivation` | `saCancelActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=finishActivation` | `saFinishActivation` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getActiveActivations` | `saGetActiveActivations` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getAllSms` | `getAllSms` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getBalance` | `getBalance` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getCountries` | `getCountries` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getHistory` | `getHistory` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getNumber` | `getNumber` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getNumberV2` | `getNumberV2` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getOperators` | `getOperators` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getPrices` | `getPrices` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getRentNumber` | `getRentNumber` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getRentServicesAndCountries` | `getRentServicesAndCountries` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getServicesList` | `getServicesList` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getStatus` | `getStatus` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getStatusV2` | `getStatusV2` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getTopCountriesByService` | `getTopCountriesByService` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=getTopCountriesByServiceRank` | `getTopCountriesByServiceRank` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `?action=prolong` | `prolong` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=prolongHistory` | `prolongHistory` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=prolongOptions` | `saProlongOptions` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | POST | `?action=reactivate` | `reactivate` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=reactivateOptions` | `saReactivateOptions` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=serviceCountRent` | `serviceCountRent` | Generic CLI `call` via skill/scripts/herosms.py |
| yes | GET | `?action=setStatus` | `setStatus` | Generic CLI `call` via skill/scripts/herosms.py |

## Classifiers

| yes | GET | `/classifiers/activations/custom-durations` | `customDurations` | Generic CLI `call` via skill/scripts/herosms.py |

## Statistics

| yes | GET | `/stats/deliverability` | `60679e939ef779f9063f3a035599e820` | Generic CLI `call` via skill/scripts/herosms.py |

## Webhooks

| yes | POST | `webhook:sms-incoming` | `smsIncoming` | Documented inbound webhook (configure URLs in HeroSMS account; not callable via CLI) |

## Summary

- **Total operations:** 51
- **CLI entrypoint:** `python skill/scripts/herosms.py`
- **Known gaps:** None beyond OpenAPI; webhook delivery is account-configured, not an outbound API.
