[
  {
    "cenário" = "Acesso ao site exemplo";
    "navegação" = [
      {
        "descrição" = ''
          O primeiro passo do cenário consiste em acessar a página de exemplo
          para confirmar que o site está acessível.
        '';
      }
      {
        "navegar para" = "exemplo.html";
      }
      {
        "capturar tela" = "exemplo.png";
        "hash esperado" = "2c1aa09c0779fbfc";
      }
    ];
  }
  {
    "cenário" = "Envio de formulário e IndexedDB";
    "navegação" = [
      {
        "descrição" = ''
          O primeiro passo do cenário consiste em acessar a página do
          formulário, preencher os dados do usuário e aguardar a mensagem de
          sucesso. Isso garante que os dados foram submetidos e salvos
          corretamente no IndexedDB.
        '';
      }
      {
        "navegar para" = "formulario.html";
      }
      {
        "enviar formulário" = {
          nome = "Fulano de Tal";
        };
      }
      {
        "esperar aparecer" = "Salvo com sucesso";
      }
      {
        "capturar tela" = "formulário-salvo.png";
        "hash esperado" = "af2631dc12c3dc67";
      }
      {
        "descrição" = ''
          Após a inserção, o teste verifica a leitura dos dados. Para isso,
          navegamos para a página de exibição e aguardamos a conclusão do
          carregamento para confirmar que os dados do IndexedDB foram
          renderizados com sucesso na tela.
        '';
      }
      {
        "navegar para" = "exibicao.html";
      }
      {
        "esperar sumir" = "Carregando";
      }
      {
        "capturar tela" = "exibição-dados.png";
        "hash esperado" = "c4b680831d8761cc";
      }
    ];
  }
]
