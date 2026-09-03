import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import zhTW from './locales/zh-TW.json'
import en from './locales/en.json'
import deDE from './locales/de-DE.json'
import frFR from './locales/fr-FR.json'
import itIT from './locales/it-IT.json'
import jaJP from './locales/ja-JP.json'
import koKR from './locales/ko-KR.json'
import koKP from './locales/ko-KP.json'
import nlNL from './locales/nl-NL.json'
import ruRU from './locales/ru-RU.json'

const messages = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  'en': en,
  'de-DE': deDE,
  'fr-FR': frFR,
  'it-IT': itIT,
  'ja-JP': jaJP,
  'ko-KR': koKR,
  'ko-KP': koKP,
  'nl-NL': nlNL,
  'ru-RU': ruRU
}

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages
})

export default i18n
