(function () {
  "use strict";

  const CSS = `
:host { display: block; width: 100%; color: var(--text, #e8edf5); font-family: inherit; box-sizing: border-box; }
* { box-sizing: border-box; }
.panel-container { width: 100%; max-width: 920px; margin: 0 auto; display: flex; flex-direction: column; gap: 16px; }
.header-card {
  display: flex; align-items: center; justify-content: space-between; padding: 16px 20px;
  background: var(--surface, rgba(255, 255, 255, 0.035)); border: 1px solid var(--border, rgba(255, 255, 255, 0.1));
  border-radius: var(--radius, 12px);
}
.title-wrap { display: flex; align-items: center; gap: 12px; }
.icon-box {
  width: 40px; height: 40px; border-radius: 10px; background: rgba(var(--accent-rgb, 110, 168, 254), 0.15);
  color: var(--accent, #6ea8fe); display: grid; place-items: center; font-size: 20px;
}
.title { font-size: 16px; font-weight: 700; color: var(--text, #e8edf5); }
.subtitle { font-size: 12px; color: var(--text-faint, #96a3b8); margin-top: 2px; }
.badge {
  display: inline-flex; align-items: center; padding: 4px 10px; border-radius: 99px; font-size: 11px;
  font-weight: 600; background: rgba(101, 211, 145, 0.12); color: #65d391; border: 1px solid rgba(101, 211, 145, 0.25);
}
.field-card {
  display: flex; flex-direction: column; gap: 10px; background: var(--surface, rgba(255, 255, 255, 0.035));
  border: 1px solid var(--border, rgba(255, 255, 255, 0.1)); border-radius: var(--radius, 12px); padding: 16px;
}
.label { font-size: 11px; font-weight: 700; color: var(--text-dim, #94a3b8); text-transform: uppercase; letter-spacing: 0.06em; }
.textarea {
  width: 100%; border: 1px solid var(--border, rgba(255, 255, 255, 0.14)); border-radius: var(--radius-sm, 8px);
  background: var(--bg, rgba(0, 0, 0, 0.25)); color: inherit; padding: 10px 12px; font: inherit; font-size: 13px; outline: none; min-height: 90px;
}
.btn-primary {
  width: 100%; padding: 12px; background: var(--accent, #6ea8fe); color: #0b101b; border: none;
  border-radius: var(--radius-sm, 8px); font-weight: 700; font-size: 14px; cursor: pointer;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.error-card {
  padding: 14px 16px; background: rgba(240, 100, 100, 0.08); border: 1px solid rgba(240, 100, 100, 0.25);
  border-radius: var(--radius, 12px); color: #f08a8a; font-size: 13px; line-height: 1.5;
}
.result-meta { font-size: 11px; color: var(--text-faint, #96a3b8); margin-top: 6px; }
`;

  class LocarynTextAnalysisPanel extends HTMLElement {
    constructor() {
      super();
      this.attachShadow({ mode: "open" });
      this.text = "";
      this.isAnalyzing = false;
      this.result = null;
      this.error = null;
    }
    connectedCallback() { this.render(); }

    async analyze() {
      if (!this.text.trim() || this.isAnalyzing) return;
      this.isAnalyzing = true;
      this.error = null;
      this.result = null;
      this.render();
      try {
        const bridge = window.locaryn || window.LocarynPluginAPI;
        if (!bridge || !bridge.invokeExtensionTool) {
          throw new Error("Le pont d'extension n'est pas disponible dans ce contexte.");
        }
        const res = await bridge.invokeExtensionTool("analyze_sentiment", { text: this.text });
        const parsed = typeof res === "string" ? JSON.parse(res) : res;
        if (!parsed || typeof parsed.sentiment !== "string" || !parsed.sentiment) {
          throw new Error("Le moteur n'a rendu aucune tonalité.");
        }
        this.result = parsed;
      } catch (err) {
        this.error = err && err.message ? err.message : String(err);
      } finally {
        this.isAnalyzing = false;
        this.render();
      }
    }

    render() {
      this.shadowRoot.innerHTML = `
        <style>${CSS}</style>
        <div class="panel-container">
          <div class="header-card">
            <div class="title-wrap">
              <div class="icon-box">📊</div>
              <div>
                <div class="title">Studio Analyse Sémantique</div>
                <div class="subtitle">Classification de tonalité par le modèle de langage local</div>
              </div>
            </div>
            <div class="badge">Actif</div>
          </div>

          <div class="field-card">
            <label class="label">Texte à analyser</label>
            <textarea class="textarea" id="ta-text" placeholder="Entrez un avis, une critique ou un document pour classifier sa tonalité...">${this.text}</textarea>
          </div>

          <button class="btn-primary" id="ta-btn" ${this.isAnalyzing || !this.text.trim() ? "disabled" : ""}>
            ${this.isAnalyzing ? "Analyse en cours..." : "Analyser le texte"}
          </button>

          ${this.error ? `<div class="error-card">${this.error}</div>` : ""}

          ${this.result ? `
            <div class="field-card" style="margin-top: 10px;">
              <div style="font-size: 14px; font-weight: 700; color: var(--accent);">
                Tonalité ${this.result.sentiment} (${(Number(this.result.score || 0) * 100).toFixed(0)}%)
              </div>
              <div class="result-meta">
                Ce pourcentage est la certitude du modèle dans sa propre réponse, pas un score
                calibré par une classification dédiée : aucun modèle de classification n'est
                embarqué dans ce morph.
              </div>
            </div>
          ` : ""}
        </div>
      `;

      const textEl = this.shadowRoot.querySelector("#ta-text");
      if (textEl) {
        textEl.addEventListener("input", (e) => {
          this.text = e.target.value;
          const btn = this.shadowRoot.querySelector("#ta-btn");
          if (btn) btn.disabled = !this.text.trim() || this.isAnalyzing;
        });
      }

      const btn = this.shadowRoot.querySelector("#ta-btn");
      if (btn) btn.addEventListener("click", () => this.analyze());
    }
  }

  if (!customElements.get("locaryn-text-analysis-panel")) {
    customElements.define("locaryn-text-analysis-panel", LocarynTextAnalysisPanel);
  }
})();
