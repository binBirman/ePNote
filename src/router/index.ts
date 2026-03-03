import { createRouter, createWebHistory } from 'vue-router'
import { initialized } from '../mock/data'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/init'
    },
    {
      path: '/init',
      name: 'init',
      component: () => import('../views/InitView.vue'),
    },
    {
      path: '/review',
      name: 'review',
      component: () => import('../views/ReviewView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/review/session',
      name: 'review-session',
      component: () => import('../views/ReviewSessionView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/questions',
      name: 'questions',
      component: () => import('../views/QuestionsView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/questions/:id',
      name: 'question-detail',
      component: () => import('../views/QuestionDetailView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/questions/new',
      name: 'question-new',
      component: () => import('../views/QuestionNewView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/stats',
      name: 'stats',
      component: () => import('../views/StatsView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/recycle-bin',
      name: 'recycle-bin',
      component: () => import('../views/RecycleBinView.vue'),
      meta: { requiresInit: true }
    },
    {
      path: '/recycle-bin/:id',
      name: 'recycle-bin-detail',
      component: () => import('../views/RecycleBinDetailView.vue'),
      meta: { requiresInit: true }
    },
  ],
})

// 路由守卫：检查初始化状态
router.beforeEach((to, from, next) => {
  if (to.meta.requiresInit && !initialized) {
    next({ name: 'init' })
  } else {
    next()
  }
})

export default router
