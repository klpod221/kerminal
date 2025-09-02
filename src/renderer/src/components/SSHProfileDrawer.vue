<template>
  <Drawer
    :visible="visible"
    title="SSH Profiles"
    position="left"
    :icon="Server"
    icon-background="bg-orange-500/20"
    icon-color="text-orange-400"
    @update:visible="$emit('update:visible', $event)"
  >
    <!-- Header Actions -->
    <div class="p-4 border-b border-gray-700">
      <!-- Search -->
      <Input
        v-model="searchQuery"
        type="text"
        placeholder="Search SSH profiles..."
        class="mb-2"
        :left-icon="Search"
        :helper="false"
      />

      <!-- Filter Toggle -->
      <div class="flex items-center justify-between">
        <Checkbox v-model="showFavoritesOnly" label="Favorites only" />

        <Button
          title="Refresh profiles"
          variant="ghost"
          size="sm"
          :icon="RefreshCw"
          @click="refreshProfiles"
        />
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Loading State -->
      <div v-if="isLoading" class="p-6 text-center">
        <div
          class="animate-spin rounded-full h-8 w-8 border-2 border-gray-600 border-t-orange-400 mx-auto mb-3"
        ></div>
        <p class="text-gray-400">Loading SSH profiles...</p>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="filteredProfiles.length === 0 && groupsWithProfiles.length === 0"
        class="p-6 text-center"
      >
        <Server :size="48" class="mx-auto mb-4 text-gray-500" />
        <h3 class="text-lg font-medium text-white mb-2">No SSH Profiles</h3>
        <p class="text-gray-400 mb-4">Create your first SSH profile to get started.</p>
        <Button variant="primary" @click="createNewProfile">
          <Plus :size="16" class="inline mr-2" />
          Create Profile
        </Button>
      </div>

      <!-- No Search Results -->
      <div
        v-else-if="searchQuery && filteredProfiles.length === 0 && filteredGroups.length === 0"
        class="p-6 text-center"
      >
        <SearchX :size="48" class="mx-auto mb-4 text-gray-500" />
        <h3 class="text-lg font-medium text-white mb-2">No Results Found</h3>
        <p class="text-gray-400">Try adjusting your search terms.</p>
      </div>

      <!-- Profiles List -->
      <div v-else class="p-4 space-y-4">
        <!-- Grouped Profiles -->
        <div v-for="group in filteredGroups" :key="group.id" class="space-y-2">
          <!-- Group Header -->
          <div class="flex items-center justify-between pt-2 group">
            <div class="flex items-center space-x-2">
              <div
                class="w-3 h-3 rounded-full"
                :style="{ backgroundColor: group.color || '#6b7280' }"
              ></div>
              <h3 class="text-sm font-medium text-white">{{ group.name }}</h3>
              <span class="text-xs text-gray-400">({{ group.profiles.length }})</span>
            </div>
            <div
              class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Button
                title="Add profile to group"
                variant="ghost"
                size="sm"
                :icon="Plus"
                @click.stop="createProfileInGroup(group)"
              />
              <Button
                title="Edit group"
                variant="ghost"
                size="sm"
                :icon="Edit3"
                @click.stop="editGroup(group)"
              />
              <PopConfirm
                :title="`Delete group '${group.name}'?`"
                content="This will not delete the profiles in the group."
                placement="bottom"
                @confirm="deleteGroup(group)"
              >
                <Button title="Delete group" variant="ghost" size="sm" :icon="Trash2" />
              </PopConfirm>
            </div>
          </div>

          <!-- Group Profiles -->
          <div class="space-y-1 ml-5">
            <!-- Show message for empty groups -->
            <div
              v-if="group.profiles.length === 0"
              class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg"
            >
              No profiles in this group. Click the + button above to add one.
            </div>

            <!-- Group profiles -->
            <div
              v-for="profile in group.profiles"
              :key="profile.id"
              class="group flex items-center justify-between p-3 bg-[#2a2a2a] hover:bg-[#333333] hover:border-gray-500 border border-transparent rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] hover:shadow-lg"
              @click="connectToProfile(profile)"
            >
              <div class="flex items-center space-x-3 flex-1 min-w-0">
                <div class="flex-shrink-0">
                  <div
                    class="w-2 h-2 rounded-full transition-all duration-300 group-hover:w-3 group-hover:h-3"
                    :style="{ backgroundColor: profile.color || group.color || '#6b7280' }"
                  ></div>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-2">
                    <p
                      class="text-sm font-medium text-white group-hover:text-blue-300 truncate transition-colors duration-300"
                    >
                      {{ profile.name }}
                    </p>
                    <Heart
                      v-if="profile.favorite"
                      :size="12"
                      class="text-red-400 fill-current flex-shrink-0 group-hover:text-red-300 transition-colors duration-300"
                    />
                  </div>
                  <p
                    class="text-xs text-gray-400 group-hover:text-gray-300 truncate transition-colors duration-300"
                  >
                    {{ profile.resolvedConfig.user }}@{{ profile.resolvedConfig.host }}:{{
                      profile.resolvedConfig.port
                    }}
                  </p>
                  <p v-if="profile.lastConnected" class="text-xs text-gray-500">
                    Last: {{ formatRelativeTime(profile.lastConnected) }}
                  </p>
                </div>
              </div>

              <div
                class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-all duration-300"
              >
                <Button
                  title="Toggle favorite"
                  variant="ghost"
                  size="sm"
                  :icon="Heart"
                  @click.stop="toggleFavorite(profile)"
                />
                <Button
                  title="Edit profile"
                  variant="ghost"
                  size="sm"
                  :icon="Edit3"
                  @click.stop="editProfile(profile)"
                />
                <PopConfirm
                  :title="`Delete profile '${profile.name}'?`"
                  content="This action cannot be undone."
                  placement="bottom"
                  @confirm="deleteProfile(profile)"
                >
                  <Button title="Delete profile" variant="ghost" size="sm" :icon="Trash2" />
                </PopConfirm>
              </div>
            </div>
          </div>
        </div>

        <!-- Ungrouped Profiles -->
        <div v-if="filteredUngroupedProfiles.length > 0" class="space-y-2">
          <div class="flex items-center justify-between py-2">
            <h3 class="text-sm font-medium text-gray-400">Ungrouped</h3>
            <span class="text-xs text-gray-500">({{ filteredUngroupedProfiles.length }})</span>
          </div>

          <div class="space-y-1">
            <div
              v-for="profile in filteredUngroupedProfiles"
              :key="profile.id"
              class="group flex items-center justify-between p-3 bg-[#2a2a2a] hover:bg-[#333333] hover:border-gray-500 border border-transparent rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] hover:shadow-lg"
              @click="connectToProfile(profile)"
            >
              <div class="flex items-center space-x-3 flex-1 min-w-0">
                <div class="flex-shrink-0">
                  <div
                    class="w-2 h-2 rounded-full transition-all duration-300 group-hover:w-3 group-hover:h-3"
                    :style="{ backgroundColor: profile.color || '#6b7280' }"
                  ></div>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-2">
                    <p
                      class="text-sm font-medium text-white group-hover:text-blue-300 truncate transition-colors duration-300"
                    >
                      {{ profile.name }}
                    </p>
                    <Heart
                      v-if="profile.favorite"
                      :size="12"
                      class="text-red-400 fill-current flex-shrink-0 group-hover:text-red-300 transition-colors duration-300"
                    />
                  </div>
                  <p
                    class="text-xs text-gray-400 group-hover:text-gray-300 truncate transition-colors duration-300"
                  >
                    {{ profile.resolvedConfig.user }}@{{ profile.resolvedConfig.host }}:{{
                      profile.resolvedConfig.port
                    }}
                  </p>
                  <p v-if="profile.lastConnected" class="text-xs text-gray-500">
                    Last: {{ formatRelativeTime(profile.lastConnected) }}
                  </p>
                </div>
              </div>

              <div
                class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-all duration-300"
              >
                <Button
                  title="Toggle favorite"
                  variant="ghost"
                  size="sm"
                  :icon="Heart"
                  @click.stop="toggleFavorite(profile)"
                />
                <Button
                  title="Edit profile"
                  variant="ghost"
                  size="sm"
                  :icon="Edit3"
                  @click.stop="editProfile(profile)"
                />
                <PopConfirm
                  :title="`Delete profile '${profile.name}'?`"
                  content="This action cannot be undone."
                  placement="bottom"
                  @confirm="deleteProfile(profile)"
                >
                  <Button title="Delete profile" variant="ghost" size="sm" :icon="Trash2" />
                </PopConfirm>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center">
        <Button
          variant="ghost"
          size="sm"
          :icon="FolderPlus"
          text="Create Group"
          @click="openManageGroups"
        />

        <Button
          variant="warning"
          size="sm"
          :icon="Plus"
          text="New Profile"
          @click="createNewProfile"
        />
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  Server,
  Search,
  RefreshCw,
  Plus,
  SearchX,
  Heart,
  Edit3,
  FolderPlus,
  Trash2
} from 'lucide-vue-next'
import Drawer from './ui/Drawer.vue'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import Checkbox from './ui/Checkbox.vue'
import PopConfirm from './ui/PopConfirm.vue'
import { formatRelativeTime } from '../utils/formatter'
import type { SSHGroupWithProfiles, SSHProfileWithConfig } from '../types/ssh'

interface Props {
  visible?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: false
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'connect-profile': [profile: SSHProfileWithConfig]
  'edit-profile': [profile: SSHProfileWithConfig]
  'create-profile': []
  'create-profile-in-group': [group: SSHGroupWithProfiles]
  'manage-groups': []
  'edit-group': [group: SSHGroupWithProfiles]
  'delete-group': [group: SSHGroupWithProfiles]
  'delete-profile': [profile: SSHProfileWithConfig]
}>()

// State
const isLoading = ref(true)
const isRefreshing = ref(false)
const searchQuery = ref('')
const showFavoritesOnly = ref(false)
const groupsWithProfiles = ref<SSHGroupWithProfiles[]>([])
const ungroupedProfiles = ref<SSHProfileWithConfig[]>([])

// Computed
const filteredGroups = computed(() => {
  if (searchQuery.value || showFavoritesOnly.value) {
    // When searching or filtering favorites, only show groups with matching profiles
    return groupsWithProfiles.value
      .map((group) => ({
        ...group,
        profiles: group.profiles.filter((profile) => {
          const matchesSearch =
            !searchQuery.value ||
            profile.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
            profile.host.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
            profile.user.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
            profile.description?.toLowerCase().includes(searchQuery.value.toLowerCase())

          const matchesFavorite = !showFavoritesOnly.value || profile.favorite

          return matchesSearch && matchesFavorite
        })
      }))
      .filter((group) => group.profiles.length > 0)
  } else {
    // When no search/filter, show all groups including empty ones
    return groupsWithProfiles.value
  }
})

const filteredUngroupedProfiles = computed(() => {
  return ungroupedProfiles.value.filter((profile) => {
    const matchesSearch =
      !searchQuery.value ||
      profile.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      profile.host.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      profile.user.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      profile.description?.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesFavorite = !showFavoritesOnly.value || profile.favorite

    return matchesSearch && matchesFavorite
  })
})

const filteredProfiles = computed(() => {
  return [
    ...filteredGroups.value.flatMap((group) => group.profiles),
    ...filteredUngroupedProfiles.value
  ]
})

// Methods
const loadProfiles = async (): Promise<void> => {
  try {
    isLoading.value = true

    const [groups, ungrouped] = await Promise.all([
      window.api.invoke('ssh-profiles.getGroupsWithProfiles'),
      window.api.invoke('ssh-profiles.getUngrouped')
    ])

    groupsWithProfiles.value = (groups as SSHGroupWithProfiles[]).map(
      (group: SSHGroupWithProfiles) => ({
        ...group,
        created: new Date(group.created),
        updated: new Date(group.updated),
        profiles: group.profiles.map((profile: SSHProfileWithConfig) => ({
          ...profile,
          created: new Date(profile.created),
          updated: new Date(profile.updated),
          lastConnected: profile.lastConnected ? new Date(profile.lastConnected) : undefined
        }))
      })
    )

    ungroupedProfiles.value = (ungrouped as SSHProfileWithConfig[]).map(
      (profile: SSHProfileWithConfig) => ({
        ...profile,
        created: new Date(profile.created),
        updated: new Date(profile.updated),
        lastConnected: profile.lastConnected ? new Date(profile.lastConnected) : undefined
      })
    )
  } catch (error) {
    console.error('Failed to load SSH profiles:', error)
  } finally {
    isLoading.value = false
  }
}

const refreshProfiles = async (): Promise<void> => {
  isRefreshing.value = true
  try {
    await loadProfiles()
  } finally {
    isRefreshing.value = false
  }
}

const connectToProfile = (profile: SSHProfileWithConfig): void => {
  emit('connect-profile', profile)
}

const editProfile = (profile: SSHProfileWithConfig): void => {
  emit('edit-profile', profile)
}

const createNewProfile = (): void => {
  emit('create-profile')
}

const createProfileInGroup = (group: SSHGroupWithProfiles): void => {
  emit('create-profile-in-group', group)
}

const openManageGroups = (): void => {
  emit('manage-groups')
}

const editGroup = (group: SSHGroupWithProfiles): void => {
  emit('edit-group', group)
}

const deleteGroup = (group: SSHGroupWithProfiles): void => {
  emit('delete-group', group)
}

const deleteProfile = (profile: SSHProfileWithConfig): void => {
  emit('delete-profile', profile)
}

const toggleFavorite = async (profile: SSHProfileWithConfig): Promise<void> => {
  try {
    await window.api.invoke('ssh-profiles.toggleFavorite', profile.id)
    // Update local state
    profile.favorite = !profile.favorite
  } catch (error) {
    console.error('Failed to toggle favorite:', error)
  }
}

// Lifecycle
onMounted(() => {
  loadProfiles()
})

// Watch for drawer open/close to refresh data
watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      refreshProfiles()
    }
  }
)

// Expose methods
defineExpose({
  refreshProfiles
})
</script>
