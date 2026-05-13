# ADR-0008: LLM local-first via Ollama; APIs externas opt-in com warning

- **Status:** Accepted
- **Data:** 2026-05-13
- **Inputs:** T1 §3.2, persona A (Marina exige on-prem)

## Contexto
Engagement contém dados sensíveis do cliente (IPs internos, credenciais,
findings). Enviar isso para API LLM pública (Anthropic, OpenAI) é:
- Risco contratual (cláusulas de não-divulgação).
- Risco de treinamento (mesmo com opt-out, "best effort").
- Risco geográfico (LGPD, soberania de dados).

T3 (xOffense) demonstra que fine-tuned 32B local supera 405B remoto em
sub-task success — performance não é mais justificativa para preferir cloud.

## Decisão
1. Backend default: **Ollama** local (`http://localhost:11434`). Modelo
   sugerido: `qwen3:32b` ou `deepseek-r1:32b` (decisão final em card próprio).
2. Backends remotos (`anthropic`, `openai`) são **opt-in** via flag
   `--llm <provider>` ou `KRI0K_LLM=anthropic:...`.
3. Ao usar backend remoto, imprime em vermelho:
   ```
   ⚠ Backend LLM remoto ativo. Snapshot do engagement será enviado para
     <provider>. Confirme conformidade com a RoE.
   ```
4. README documenta como rodar 100% air-gapped.

## Consequências
- ✅ Persona Marina (banco) consegue usar.
- ✅ Persona Heitor (acadêmico) consegue reproduzir sem custo de API.
- ❌ Setup inicial é mais pesado (baixar modelo de ~20GB); mitigação:
  README com script + alternativa de modelo menor para smoke test.

## Alternativas consideradas
- **vLLM/SGLang como default:** mais throughput, mas Ollama tem UX melhor
  para dev solo; manter como opção em ADR futuro.
- **Bring-your-own-LLM via OpenAI-compatible API:** já coberto por `--llm
  openai:` apontando para localhost.
