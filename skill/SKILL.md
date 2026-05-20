---
name: fln
description: Falsifier Ledger Network 영구 의사결정 ledger. 사용자가 *의사결정의 thesis + falsifier + 인과 그래프* 를 한 번에 등록·서명·검증·평가·앵커링하려 할 때 활성화. `fln` CLI / `fln-mcp` MCP 서버 / `fln-oracle` 시장-데이터 평가기를 묶어 단일 흐름으로 안내.

자동 로드 트리거:
(a) 사용자가 "thesis / falsifier / 가설 / 인과 / causal / ledger / 의사결정 / decision journal / 사후검증 / 폐기 조건" 키워드 + 진입 결정·전략·실험 컨텍스트,
(b) `fln` 도구 (CLI/MCP) 사용 의사 명시 ("/fln", "fln-mcp", "fln-oracle", "anchor"),
(c) 사용자가 finance-journal/crypto-scenarist 흐름에서 *영속화* 단계로 진입,
(d) ledger / merkle root / 앵커 / 공증 / 영구 기록 / 비가역 ledger 키워드,
(e) /fln 슬래시 명령.

SKIP: 단순 "thesis" 만으로 매칭 거부 — "falsifier / 폐기조건 / ledger / 영구기록" 등 FLN 의 영속·서명 요구가 명시될 때만. 1회성 가설 토론 / 분석은 거부.
---

# FLN — Falsifier Ledger Network 사용 가이드

영구 ledger 에 의사결정을 *영속·서명·평가* 한다. 모든 의사결정에 Popper falsifier + Pearl 인과 DAG + Bayesian decay weight 를 기계 검증 가능한 형식으로 첨부한다.

## 언제 사용하는가

1. **진입 결정** (투자/실험/정책/연구 가설) 직전 — thesis + falsifier 를 ledger 에 영속화한 후에만 실제 행동.
2. **사후 검증** — 30/90/180일 후 falsifier 평가, decay weight 업데이트, 사후 회고 시 ledger root 로 시점-증명.
3. **공증** — 6 시간/일 단위로 ledger root 를 anchor 로 묶어 외부 (GitHub Pages / OTS / 공개 git) 에 publish.

## 표준 흐름

### 1) thesis 등록 (서명 전)

```bash
fln thesis-new --id <id> --domain <invest|health|real_estate|policy|science|engineering> \
    --claim "<one-line hypothesis>" --out theses/<id>.thesis.json
fln causal-add-node --thesis theses/<id>.thesis.json --id <var> --label "<label>" --kind <cause|effect|confounder|mediator>
fln causal-add-edge --thesis theses/<id>.thesis.json --from <a> --to <b>
fln causal-topo --thesis theses/<id>.thesis.json
```

### 2) Falsifier 구조화 — 평가 가능 형식

`theses/<id>.predicates.json` 추가:

```json
{
  "thesis_id": "<id>",
  "predicates": [
    {
      "falsifier_idx": 0, "ticker": "BTC-USD", "field": "close",
      "op": "lt", "rhs": 80000,
      "window": {"kind": "any_close", "lookback_days": 90}
    }
  ]
}
```

### 3) 서명 + ledger 영속

```bash
fln key-new --out keys/<name>            # 최초 1회
fln thesis-sign --thesis theses/<id>.thesis.json --sk keys/<name>.sk --out theses/<id>.claim.json
fln thesis-verify --claim theses/<id>.claim.json
fln ledger-append --ledger ledger.json --thesis theses/<id>.thesis.json
```

### 4) 평가 (사후 또는 주기적)

```bash
fln-oracle evaluate --predicates theses/<id>.predicates.json --out theses/<id>.evaluation.json
# exit code 2 ⇔ 적어도 하나의 falsifier triggered
```

평가 결과를 보고 `decay-update` 로 weight 업데이트:

```bash
fln decay-update --thesis theses/<id>.thesis.json \
    --delta-days <days> --outcome <-1..+1> --regime-signal <VIX 등>
```

### 5) 앵커 발급

```bash
fln anchor --ledger ledger.json --sk keys/<name>.sk --out anchors/$(date +%Y-%m-%d).anchor.json
fln anchor-verify --anchor anchors/<date>.anchor.json
```

## MCP 서버

stdio 로 동일 작업을 LLM 에이전트가 수행할 수 있다. 등록:

```jsonc
// ~/.claude.json
{ "mcpServers": { "fln": { "command": "fln-mcp" } } }
```

도구: `create_thesis`, `add_falsifier`, `add_causal_node`, `add_causal_edge`,
`causal_topo`, `generate_key`, `sign_thesis`, `append_ledger`, `decay_update`,
`get_thesis`. 상태는 `$FLN_STATE_DIR` (기본 `~/.fln`) 에 영속.

## 도메인별 default τ (반감기, 일)

| 도메인 | τ |
|---|---|
| invest | 180 |
| health | 730 |
| real_estate | 365 |
| policy | 365 |
| science | 1825 |
| engineering | 365 |

## 안티 패턴

- Falsifier 없이 thesis 등록 → ledger 가치 0.
- `claim` 에 "할 수도 있다" 같은 비반증 가능 표현 → 거부, 다시 작성.
- 동일 시점 다중 falsifier 묶기 → 분리해서 별도 thesis 로.
- 서명 키 평문 commit → 즉시 회전.

## 관련 RDU / glossary

- `~/.claude/rules/glossary-trading.md` — EntryThesis / Falsifier / BaseRate 단어 일관성
- RDU-021 / RDU-022 / RDU-023 — calibration / walkforward / base rate
- RDU-034 — pre-trade gate
- `finance-journal` skill — Layer 3 prototype (사용자 운영 중)

## 구현 참조

- Rust core: `/Volumes/EXT_SSD/bot/fln/crates/fln-core/`
- Python ref: `/Volumes/EXT_SSD/bot/fln/python/fln/`
- IETF draft: `/Volumes/EXT_SSD/bot/fln/ietf/draft-fln-falsifier-ledger-00.md`
