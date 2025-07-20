# vocab_analyser

Ferramenta web para análise de vocabulário a partir de textos.  
Permite ao usuário marcar palavras como conhecidas ou desconhecidas, com persistência por usuário.

> ⚠️ Projeto em estágio inicial.  
> Atualmente com um protótipo de backend em Rust e uso de `user_id` fixo para testes locais.

---

## Objetivo

Facilitar o aprendizado de vocabulário em contexto.  
Permitir que o usuário leia textos completos e marque palavras que já conhece, auxiliando no estudo ativo de línguas e análise lexical personalizada.

---

## Stack

- Rust (Axum, tokio)
- PostgreSQL
- JavaScript no frontend

---

## Funcionalidades atuais

- Tokenização simples
- Marcação de palavras como conhecidas/desconhecidas
- Persistência com PostgreSQL
- Backend em Rust usando `Axum`

---

## Em desenvolvimento

- Autenticação real de usuários
- API REST acessível por frontend
- Interface web interativa

---

## Como executar

**Pré-requisitos:**

- Rust (`cargo`)
- PostgreSQL

**Passos básicos:**

```bash
git clone https://github.com/DanielLourencoJr/vocab_analyser
cd vocab_analyser
cargo run

