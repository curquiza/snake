require 'yaml/store'

class Database
  DATABASE = YAML::Store.new('score.yaml')

  def self.save_score(score_data)
    DATABASE.transaction do
      DATABASE[:scores] ||= []
      DATABASE[:scores] << score_data
    end
  end

  def self.get_score(index)
    DATABASE.transaction do
        DATABASE[:scores]&[index]
    end
  end

end


score_data = {
    username: 'Clem',
    score: 21,
    created_date: Time.now
}

# Database.save_score score_data
score = Database.get_score(12)
puts(score&[:username] || 'No data at this index')
