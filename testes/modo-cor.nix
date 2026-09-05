[
  {
    "cenário" = "Navegação com modo claro padrão";
    "navegação" = [
      {
        "descrição" = ''
          Verifica se a página renderiza em modo claro por padrão (quando a opção modo não é informada).
        '';
      }
      {
        "navegar para" = "modo-cor.html";
      }
      {
        "esperar aparecer" = "Modo Claro Ativo";
      }
      {
        "capturar tela" = "modo-cor-claro-padrao.png";
        "hash esperado" = "33d1a15ea34fd765";
      }
    ];
  }
  {
    "cenário" = "Navegação com modo claro explícito";
    "modo" = "claro";
    "navegação" = [
      {
        "descrição" = ''
          Verifica se a página renderiza em modo claro quando configurada explicitamente com modo claro.
        '';
      }
      {
        "navegar para" = "modo-cor.html";
      }
      {
        "esperar aparecer" = "Modo Claro Ativo";
      }
      {
        "capturar tela" = "modo-cor-claro-explicito.png";
        "hash esperado" = "33d1a15ea34fd765";
      }
    ];
  }
  {
    "cenário" = "Navegação com modo escuro";
    "modo" = "escuro";
    "navegação" = [
      {
        "descrição" = ''
          Verifica se a página renderiza em modo escuro quando configurada com modo escuro.
        '';
      }
      {
        "navegar para" = "modo-cor.html";
      }
      {
        "esperar aparecer" = "Modo Escuro Ativo";
      }
      {
        "capturar tela" = "modo-cor-escuro.png";
        "hash esperado" = "56758173e136bf65";
      }
    ];
  }
]
