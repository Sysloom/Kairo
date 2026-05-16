# Kairo · Logo system

Sistema completo de marca para **Kairo**. Todos los archivos son SVG vectoriales **completamente outlined** — no dependen de ninguna fuente externa, renderizan idénticamente en cualquier visor.

## Carpetas

- `wordmark/`  — el wordmark completo "kair + arco". Variantes de peso y color.
- `symbol/`    — solo el símbolo (arco cíclico). Base para favicon y system tray.
- `progress/`  — el wordmark con el arco indicando estados del timer (100/90/50/25%). Útil para mockups y animación.
- `app-icon/`  — tile redondeado con el símbolo dentro. Listo para escritorio / docks.
- `favicon/`   — favicons en escalas optimizadas (16, 32, 48, 64) y `favicon.svg` escalable.
- `lockup/`    — composición vertical wordmark + descriptor "FOCUS · POMODORO · CYCLES".

## Defaults

- **Wordmark primario**: `wordmark/kairo-wordmark-medium.svg`
- **Símbolo primario**: `symbol/kairo-symbol-ink.svg`
- **App icon**: `app-icon/kairo-app-icon-ink.svg`
- **Favicon**: `favicon/favicon.svg` (scalable)

## Tipografía

El wordmark usa **Geist Sans**, outlined a paths vectoriales. No requiere instalación. Pesos disponibles: Light 300, Regular 400, Medium 500 (default), SemiBold 600.

Si querés generar el wordmark a partir del nombre vivo, instalá Geist y usá:

```
font-family: "Geist", system-ui;
font-weight: 500;
letter-spacing: -0.045em;
```

## Paleta

| Token   | Hex / valor              | Uso                           |
|---------|--------------------------|-------------------------------|
| ink     | `#17171C`              | foreground primario           |
| paper   | `#FBF9F4`              | fondo cálido                  |
| paper-fg| `#F4F1EB`              | foreground sobre ink          |
| accent  | `oklch(0.64 0.14 35)`  | acento opcional (terracota)   |
| accent (hex) | `#C66A48`         | fallback                      |

## Reglas

- **Clear space**: x = radio del arco. Mantener al menos x alrededor de la marca.
- **Tamaño mínimo wordmark**: 18px de altura de "kair".
- **Símbolo mínimo**: 12px (favicon-16 está pre-engordado).
- **No** estirar, rotar, cerrar el arco, ni usar ink sobre accent.
