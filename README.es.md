# Kairo

Kairo es una app de temporizador de foco local-first para **Fedora KDE / Plasma / Wayland** construida con **Tauri v2, React, TypeScript, Rust y SQLite**.

Está diseñada alrededor de una regla no negociable: **Rust es la fuente de verdad del timer y de la persistencia real de foco**.

Versión en inglés: [README.md](./README.md)

## Descargar RPM

**Build para Fedora KDE 44:**

- RPM: [Kairo-0.1.2-1.x86_64.rpm](https://github.com/Sysloom/Kairo/releases/download/v0.1.2/Kairo-0.1.2-1.x86_64.rpm)

## Camino rápido

1. Instalá el RPM en Fedora KDE 44.
2. Abrí Kairo normalmente.
3. Si estás en KDE/Wayland, Kairo instala y habilita automáticamente la integración KWin para mantener los timers flotantes por encima de otras ventanas.

## Fixes incluidos en esta versión

- las alarmas MP3 empaquetadas vuelven a reproducirse correctamente en la app instalada;
- los timers flotantes usan títulos estables para que KWin los pueda reconocer bien;
- Kairo empaqueta un script `kairo-keep-above` y lo instala/habilita automáticamente para el usuario actual en KDE/Wayland;
- el comportamiento de ventana flotante fue ajustado para que el timer quede por encima de apps nuevas cuando KWin respeta la integración.

## Instalación

### Opción recomendada: Fedora KDE 44

```bash
sudo dnf install -y "https://github.com/Sysloom/Kairo/releases/download/v0.1.2/Kairo-0.1.2-1.x86_64.rpm"
```

Si ya lo tenías instalado:

```bash
sudo dnf reinstall -y "https://github.com/Sysloom/Kairo/releases/download/v0.1.2/Kairo-0.1.2-1.x86_64.rpm"
```

## Qué incluye hoy

- integración con system tray;
- ventana principal;
- floating timer;
- mini timer para countdown con ventanas ocultas;
- persistencia local SQLite;
- flujo start / pause / resume / reset;
- tema claro y oscuro;
- alarmas MP3 empaquetadas con fallback audible.

## KDE / Wayland

- `alwaysOnTop` por sí solo es best-effort bajo Wayland;
- por eso Kairo usa una integración KWin desde el lado del compositor;
- el RPM ya incluye esa integración y Kairo la prepara automáticamente para el usuario actual al arrancar;
- fallback manual para builds locales o troubleshooting:

```bash
npx pnpm@10.11.0 kde:install-kwin-script
```

## Verificación rápida

- [ ] Abrir Kairo.
- [ ] Iniciar una sesión de foco.
- [ ] Mostrar el floating timer.
- [ ] Abrir otra app encima.
- [ ] Confirmar que el timer sigue visible arriba.
- [ ] Confirmar que la alarma MP3 suena al completar.

## Archivos importantes

- `README.md` — documentación principal en inglés
- `src/services/audioService.ts` — reproducción de alarmas y fallback
- `src-tauri/src/infrastructure/windows.rs` — lifecycle de ventanas
- `src-tauri/src/infrastructure/kde_integration.rs` — integración KDE/Wayland por usuario
- `packaging/kde/kwin/kairo-keep-above/` — script KWin empaquetado
